use super::{cloud_llm, local_llm};

pub enum LlmSource {
    Local,
    Cloud,
}

pub async fn get_response(source: LlmSource, prompt: &str) -> String {
    match source {
        LlmSource::Local => local_llm::generate_response(prompt),
        LlmSource::Cloud => cloud_llm::generate_response(prompt).await,
    }
}
