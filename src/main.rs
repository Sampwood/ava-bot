mod handlers;
mod error;

use std::{sync::Arc, env::var};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use llm_sdk::LlmSdk;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[derive(Debug, Parser)]
#[clap(name = "ava")]
struct Args {
    #[clap(short, long, default_value = "8080")]
    port: u16,
    #[clap(short, long, default_value = ".certs")]
    cert_path: String,
}

#[derive(Debug)]
pub struct AppState {
    llm_sdk: LlmSdk,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            llm_sdk: LlmSdk::new_with_base_url(var("OPENAI_API_KEY").unwrap(), "https://api.openai.com/v1"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let state = Arc::new(AppState::default());
    let serve_dir = ServeDir::new("public").not_found_service(ServeFile::new("public/index.html"));
    let app = Router::new()
        .route("/api/chats", get(handlers::chats_handler))
        .route("/api/assistant", post(handlers::assistant_handler))
        .nest_service("/assets", ServeDir::new("./public/assets"))
        .fallback_service(serve_dir)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    info!("Listening on {}", addr);

    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
