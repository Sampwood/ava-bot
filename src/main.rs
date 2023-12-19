use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use clap::Parser;
use tracing::info;

#[derive(Debug, Parser)]
#[clap(name = "ava")]
struct Args {
    #[clap(short, long, default_value = "8080")]
    port: u16,
    #[clap(short, long, default_value = ".certs")]
    cert_path: String,
}

#[derive(Debug, Default)]
pub(crate) struct AppState {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let state = Arc::new(AppState::default());
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    info!("Listening on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
