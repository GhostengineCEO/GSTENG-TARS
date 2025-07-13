use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

use super::cloud_llm;

static CURRENT_MODEL: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("llama2".to_string()));

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateChunk {
    response: Option<String>,
    done: bool,
}

/// Attempt to generate a response from the local Ollama instance. If the call
/// fails, the function falls back to the cloud model.
pub async fn generate_response(prompt: &str) -> String {
    let model = { CURRENT_MODEL.read().await.clone() };
    match generate_with_model(&model, prompt).await {
        Ok(r) => r,
        Err(_) => cloud_llm::generate_response(prompt).await,
    }
}

async fn generate_with_model(model: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let req = GenerateRequest {
        model,
        prompt,
        stream: true,
    };
    let resp = client
        .post("http://localhost:11434/api/generate")
        .json(&req)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(reqwest::Error::new(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "ollama",
        ));
    }

    let mut stream = resp.bytes_stream();
    let mut out = String::new();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        for line in bytes.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            if let Ok(data) = serde_json::from_slice::<GenerateChunk>(line) {
                if let Some(content) = data.response {
                    out.push_str(&content);
                }
                if data.done {
                    return Ok(out);
                }
            }
        }
    }
    Ok(out)
}

#[derive(Deserialize)]
struct ModelTags {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
}

/// Download a model using Ollama's pull API
pub async fn download_model(model: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let body = serde_json::json!({ "name": model });
    client
        .post("http://localhost:11434/api/pull")
        .json(&body)
        .send()
        .await?;
    Ok(())
}

/// Retrieve the list of locally available models
pub async fn list_models() -> Result<Vec<String>, reqwest::Error> {
    let client = Client::new();
    let resp = client.get("http://localhost:11434/api/tags").send().await?;
    let list: ModelTags = resp.json().await?;
    Ok(list.models.into_iter().map(|m| m.name).collect())
}

/// Switch the active model for future requests
pub async fn switch_model(model: &str) -> Result<(), reqwest::Error> {
    *CURRENT_MODEL.write().await = model.to_string();
    Ok(())
}
