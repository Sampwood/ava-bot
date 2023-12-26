pub mod error;
pub mod handlers;
pub mod extractors;

use std::{
    env::var,
    path::{Path, PathBuf},
};

use llm_sdk::LlmSdk;
use uuid::Uuid;

#[derive(Debug)]
pub struct AppState {
    public_path: PathBuf,
    pub(crate) llm_sdk: LlmSdk,
    // pub(crate) EventMap: 

}

impl AppState {
    pub fn new(public_path: &Path) -> Self {
        Self {
            llm_sdk: LlmSdk::new_with_base_url(
                var("OPENAI_API_KEY").unwrap(),
                "https://api.openai.com/v1",
            ),
            public_path: public_path.to_path_buf(),
        }
    }
}

pub fn audio_path(device_id: &str) -> (PathBuf, String) {
    let filename = format!("{}.mp3", Uuid::new_v4().to_string());
    let path = PathBuf::new().join("audio").join(device_id).join(&filename);
    let url = format!("/audio/{}/{}", device_id, filename);

    (path, url)
}
