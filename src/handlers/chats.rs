use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::State,
    response::{IntoResponse, Sse, sse::Event},
};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tracing::info;

use crate::{extractors::AppContext, AppState};

const MAX_EVENTS: usize = 128;

pub async fn chats_handler(
    State(state): State<Arc<AppState>>,
    ctx: AppContext,
) -> impl IntoResponse {
    info!("sse connected");

    let device_id = ctx.device_id;
    let rx = match state.senders.get(&device_id) {
        Some(sender) => sender.subscribe(),
        None => {
            let (tx, rx) = broadcast::channel(MAX_EVENTS);
            state.senders.insert(device_id, tx);
            rx
        }
    };


    // wrap receiver in a stream
    let stream = BroadcastStream::new(rx)
        .filter_map(|v| v.ok())
        .map(|v| Event::default().data(v))
        .map(Ok::<_, Infallible>);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
