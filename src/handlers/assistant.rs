use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};
use dashmap::mapref::one::Ref;
use llm_sdk::{
    ChatCompletionMessage, ChatCompletionRequest, LlmSdk, SpeechRequest, WhisperRequest,
};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::{fs, sync::broadcast::Sender};
use tracing::info;

use crate::{audio_path, error::AppError, extractors::AppContext, AppState};

pub async fn assistant_handler(
    context: AppContext,
    State(state): State<Arc<AppState>>,
    mut data: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let sender = get_sender(&state, &context.device_id)?;

    sender.send(EventType::in_audio_upload())?;
    let Some(field) = data.next_field().await.unwrap() else {
        return Err(anyhow!("expected an audio field"))?;
    };
    let data = match field.name() {
        Some("audio") => field.bytes().await?,
        _ => return Err(anyhow!("expected an audio field"))?,
    };
    let len = data.len();
    info!("Length of `audio` is {} bytes", data.len());

    sender.send(EventType::in_transcription())?;
    let req = WhisperRequest::transcription(data.to_vec());
    let res = state.llm_sdk.whisper(req).await?;
    let input = res.text;

    sender.send(EventType::in_chat_completion())?;
    let output = chat_completion(&state.llm_sdk, &input).await?;
    sender.send(EventType::in_speech())?;
    let audio_url = speech(&state, &context.device_id, &output).await?;

    sender.send(EventType::done())?;
    let res_data =
        json!({ "len": len, "request": input, "response": output, "audio_url": audio_url });
    sender.send(EventType::message(res_data))?;
    Ok(())
}

fn get_sender<'a>(state: &'a AppState, device_id: &str) -> Result<Ref<'a, String, Sender<String>>> {
    state
        .senders
        .get(device_id)
        .ok_or_else(|| anyhow!("device_id not found in senders"))
}

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

async fn speech(state: &AppState, device_id: &str, output: &str) -> Result<String> {
    let req = SpeechRequest::new(output);
    let data = state.llm_sdk.speech(req).await?;

    let (relative_path, url) = audio_path(device_id);
    let path = state.public_path.join(&relative_path);

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }
    fs::write(&path, data).await?;

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
    fn in_audio_upload() -> String {
        EventType::Signal(SignalType::UploadAudio).to_json()
    }
    fn in_transcription() -> String {
        EventType::Signal(SignalType::Transcription).to_json()
    }
    fn in_chat_completion() -> String {
        EventType::Signal(SignalType::ChatCompletion).to_json()
    }
    fn in_speech() -> String {
        EventType::Signal(SignalType::Speech).to_json()
    }
    fn done() -> String {
        EventType::Signal(SignalType::Done).to_json()
    }
}
