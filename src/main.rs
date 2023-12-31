use std::{path::Path, sync::Arc};

use anyhow::Result;
use ava_bot::{
    handlers::{assistant_handler, chats_handler, common},
    AppState,
};
use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use clap::Parser;
use tower_http::services::ServeDir;
use tracing::info;

#[derive(Debug, Parser)]
#[clap(name = "ava")]
struct Args {
    #[clap(short, long, default_value = "8080")]
    port: u16,
    #[clap(short, long, default_value = ".certs")]
    cert_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let public_path = Path::new("public");
    let state = Arc::new(AppState::new(&public_path));
    let app = Router::new()
        .route("/api/chats", get(chats_handler))
        .route("/api/assistant", post(assistant_handler))
        .layer(from_fn(common::layer_auth))
        .fallback_service(ServeDir::new(&public_path))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    info!("Listening on {}", addr);

    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
