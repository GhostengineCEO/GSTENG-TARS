use tauri::command;
use crate::ai::{router, router::LlmSource};

#[command]
pub async fn ask_ai(prompt: String, use_cloud: bool) -> String {
    let source = if use_cloud { LlmSource::Cloud } else { LlmSource::Local };
    router::get_response(source, &prompt).await
}
