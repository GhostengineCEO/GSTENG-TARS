use std::env;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

/// Structures used to communicate with the OpenAI API
#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatChunk {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

/// Generate a response from the OpenAI API. The function internally uses
/// streaming responses but returns the aggregated result as a `String`.
pub async fn generate_response(prompt: &str) -> String {
    let key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => return "OpenAI API key not set".into(),
    };

    let client = match Client::builder().timeout(Duration::from_secs(30)).build() {
        Ok(c) => c,
        Err(e) => return format!("Failed to build HTTP client: {e}"),
    };

    let request = ChatRequest {
        model: "gpt-3.5-turbo",
        messages: vec![Message {
            role: "user",
            content: prompt,
        }],
        stream: true,
    };

    let url = "https://api.openai.com/v1/chat/completions";
    // simple retry logic
    for _ in 0..3 {
        match client
            .post(url)
            .bearer_auth(&key)
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    continue;
                }
                let mut stream = response.bytes_stream();
                let mut out = String::new();
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(bytes) => {
                            for line in bytes.split(|&b| b == b'\n') {
                                let line = match line.strip_prefix(b"data: ") {
                                    Some(l) => l,
                                    None => continue,
                                };
                                if line == b"[DONE]" {
                                    return out;
                                }
                                if let Ok(chunk) = serde_json::from_slice::<ChatChunk>(line) {
                                    if let Some(content) =
                                        chunk.choices.get(0).and_then(|c| c.delta.content.clone())
                                    {
                                        out.push_str(&content);
                                    }
                                }
                            }
                        }
                        Err(_) => return "Failed to read stream".into(),
                    }
                }
                return out;
            }
            Err(_) => {
                // wait a bit before retrying
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    "Failed to contact OpenAI".into()
}
