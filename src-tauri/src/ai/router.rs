use std::collections::HashMap;
use std::env;
use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use super::{cloud_llm, local_llm};

/// Simple in-memory cache for prompts and their responses
static CACHE: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub enum LlmSource {
    Local,
    Cloud,
}

fn network_available() -> bool {
    reqwest::blocking::Client::new()
        .get("https://www.google.com")
        .timeout(Duration::from_secs(2))
        .send()
        .is_ok()
}

fn query_complexity(prompt: &str) -> usize {
    prompt.split_whitespace().count()
}

/// Route the prompt to either the local or cloud model based on heuristics.
pub async fn get_response(source: LlmSource, prompt: &str) -> String {
    // Check cache first
    if let Some(cached) = CACHE.read().await.get(prompt).cloned() {
        return cached;
    }

    // Determine which source to use
    let mut chosen = source;

    if env::var("AI_PRIVACY_LOCAL_ONLY").ok().as_deref() == Some("1") {
        chosen = LlmSource::Local;
    }

    if !network_available() {
        chosen = LlmSource::Local;
    }

    if query_complexity(prompt) > 50 {
        chosen = LlmSource::Cloud;
    }

    let result = match chosen {
        LlmSource::Local => local_llm::generate_response(prompt).await,
        LlmSource::Cloud => cloud_llm::generate_response(prompt).await,
    };

    CACHE
        .write()
        .await
        .insert(prompt.to_string(), result.clone());
    result
}
