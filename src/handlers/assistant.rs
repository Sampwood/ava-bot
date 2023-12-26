use std::{
    os::macos::raw::stat,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use llm_sdk::{
    ChatCompletionMessage, ChatCompletionRequest, LlmSdk, SpeechRequest, WhisperRequest,
};
use serde_json::json;
use tokio::fs;
use tracing::info;

use crate::{
    audio_path, error::AppError, extractors::AppContext, handlers::common::COOKIE_NAME, AppState,
};

pub async fn assistant_handler(
    context: AppContext,
    State(state): State<Arc<AppState>>,
    mut data: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let Some(field) = data.next_field().await.unwrap() else {
        return Err(anyhow!("expected an audio field"))?;
    };
    let data = match field.name() {
        Some("audio") => field.bytes().await?,
        _ => return Err(anyhow!("expected an audio field"))?,
    };
    let len = data.len();
    info!("Length of `audio` is {} bytes", data.len());

    let req = WhisperRequest::transcription(data.to_vec());
    let res = state.llm_sdk.whisper(req).await?;
    let input = res.text;

    let output = chat_completion(&state.llm_sdk, &input).await?;
    let audio_url = speech(&state, &context.device_id, &output).await?;

    Ok(Json(
        json!({ "len": len, "request": input, "response": output, "audio_url": audio_url }),
    ))
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
