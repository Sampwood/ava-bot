use std::{time::Duration, convert::Infallible};

use axum::response::{IntoResponse, sse::Event, Sse};
use futures::stream;
use tokio_stream::StreamExt as _;
use tracing::info;

pub async fn chats_handler() -> impl IntoResponse {
    info!("sse connected");

    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok::<_, Infallible>)
        .throttle(Duration::from_secs(120));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
