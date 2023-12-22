use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    extract::{Multipart, State},
    response::IntoResponse, Json,
};
use llm_sdk::WhisperRequest;
use serde_json::json;
use tracing::info;

use crate::{error::AppError, AppState};

pub async fn assistant_handler(
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
    Ok(Json(json!({ "len": len, "request": res.text, "response": "" })))
}
