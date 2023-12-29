use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};
use chrono::Local;
use dashmap::mapref::one::Ref;
use llm_sdk::{
    ChatCompletionMessage, ChatCompletionRequest, CreateImageRequest, LlmSdk, SpeechRequest, Tool,
    WhisperRequestBuilder, WhisperRequestType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{fs, sync::broadcast::Sender};

use crate::{audio_path, error::AppError, extractors::AppContext, write_file, AppState};

pub async fn assistant_handler(
    context: AppContext,
    State(state): State<Arc<AppState>>,
    mut data: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let sender = get_sender(&state, &context.device_id)?;

    sender.send(EventType::signal_audio_upload())?;
    let Some(field) = data.next_field().await.unwrap() else {
        return Err(anyhow!("expected an audio field"))?;
    };
    let data = match field.name() {
        Some("audio") => field.bytes().await?,
        _ => return Err(anyhow!("expected an audio field"))?,
    };

    sender.send(EventType::signal_transcription())?;
    let input = transcript(&state.llm_sdk, data.to_vec()).await?;

    sender.send(EventType::message_input(&input))?;

    sender.send(EventType::signal_chat_completion())?;
    chat_completion_with_tools(&state, &context.device_id, &input, &sender).await?;

    Ok(())
}

fn get_sender<'a>(state: &'a AppState, device_id: &str) -> Result<Ref<'a, String, Sender<String>>> {
    state
        .senders
        .get(device_id)
        .ok_or_else(|| anyhow!("device_id not found in senders"))
}

async fn transcript(llm_sdk: &LlmSdk, data: Vec<u8>) -> Result<String> {
    let req = WhisperRequestBuilder::default()
        .file(data)
        .prompt("If audio language is Chinese, please use Simplified Chinese")
        .request_type(WhisperRequestType::Transcription)
        .build()
        .unwrap();
    let res = llm_sdk.whisper(req).await?;
    Ok(res.text)
}

#[allow(dead_code)]
async fn chat_completion(llm_sdk: &LlmSdk, prompt: &str) -> Result<String> {
    let messages = vec![
        ChatCompletionMessage::new_system("I can choose the right function for you.", "ava"),
        ChatCompletionMessage::new_user(prompt, "user1"),
    ];
    let req = ChatCompletionRequest::new(messages);
    let mut res = llm_sdk.chat_completion(req).await?;
    let content = res
        .choices
        .pop()
        .ok_or_else(|| anyhow!("expect at least one choice"))?
        .message
        .content
        .ok_or_else(|| anyhow!("expect content but no content available"))?;
    Ok(content)
}

async fn chat_completion_with_tools(
    state: &AppState,
    device_id: &str,
    prompt: &str,
    sender: &Sender<String>,
) -> Result<String> {
    let messages = vec![
        ChatCompletionMessage::new_system("I can help to identify which tool to use, if no proper tool could be used, I'll directly reply the message with pure text", "ava"),
        ChatCompletionMessage::new_user(prompt, "user1"),
    ];
    let tools = vec![
        Tool::new_function::<DrawImageArgs>("draw_image", "Draw an image based on the prompt."),
        // Tool::new_function::<WriteCodeArgs>("write_code", "Write code based on the prompt."),
        // Tool::new_function::<AnswerArgs>("answer", "Just reply based on the prompt."),
    ];
    let req = ChatCompletionRequest::new_with_tools(messages, tools);
    let mut res = state.llm_sdk.chat_completion(req).await?;
    let mut message = res
        .choices
        .pop()
        .ok_or_else(|| anyhow!("expect at least one choice"))?
        .message;
    let function = &message
        .tool_calls
        .pop()
        .ok_or_else(|| anyhow!("expect at least one toll_call"))?
        .function;

    if function.name == "draw_image" {
        let args: DrawImageArgs = serde_json::from_str(&function.arguments)?;
        let (url, revised_prompt) = draw_image(&state.llm_sdk, &args.prompt).await?;
        sender.send(EventType::message_draw(&revised_prompt, &url))?;
    } else {
        let content = message
            .content
            .ok_or_else(|| anyhow!("expect content but no content available"))?;

        sender.send(EventType::signal_speech())?;
        let audio_url = speech(&state, device_id, &content).await?;

        sender.send(EventType::signal_done())?;
        sender.send(EventType::message_speech(&content, &audio_url))?;
    }

    Ok("".to_owned())
}

async fn draw_image(llm_sdk: &LlmSdk, prompt: &str) -> Result<(String, String)> {
    let req = CreateImageRequest::new(prompt);
    let mut res = llm_sdk.create_image(req).await?;
    let image = res.data.pop().unwrap();
    Ok((image.url.unwrap(), image.revised_prompt))
}

async fn speech(state: &AppState, device_id: &str, output: &str) -> Result<String> {
    let req = SpeechRequest::new(output);
    let data = state.llm_sdk.speech(req).await?;

    let (relative_path, url) = audio_path(device_id);
    write_file(state.public_path.join(&relative_path), data).await?;

    Ok(url)
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum EventType {
    Message(Value),
    Signal(SignalType),
}
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum SignalType {
    UploadAudio,
    Transcription,
    ChatCompletion,
    Speech,
    Done,
}

impl EventType {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn message(value: Value) -> String {
        EventType::Message(value).to_json()
    }
    fn message_input(message: &str) -> String {
        EventType::message(json!({
            "owner": "user",
            "message": message,
            "date_time": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }))
    }
    fn message_speech(message: &str, url: &str) -> String {
        EventType::message(json!({
            "owner": "ava",
            "type": "speech",
            "message": message,
            "url": url,
            "date_time": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }))
    }
    fn message_draw(revised_prompt: &str, url: &str) -> String {
        EventType::message(json!({
            "owner": "ava",
            "type": "image",
            "prompt": revised_prompt,
            "url": url,
            "date_time": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }))
    }

    fn signal_audio_upload() -> String {
        EventType::Signal(SignalType::UploadAudio).to_json()
    }
    fn signal_transcription() -> String {
        EventType::Signal(SignalType::Transcription).to_json()
    }
    fn signal_chat_completion() -> String {
        EventType::Signal(SignalType::ChatCompletion).to_json()
    }
    fn signal_speech() -> String {
        EventType::Signal(SignalType::Speech).to_json()
    }
    fn signal_done() -> String {
        EventType::Signal(SignalType::Done).to_json()
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DrawImageArgs {
    prompt: String,
}
