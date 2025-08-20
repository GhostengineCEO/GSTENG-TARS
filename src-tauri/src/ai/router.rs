use std::collections::HashMap;
use std::env;
use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use super::{cloud_llm, local_llm};
use crate::personality::{TARSCore, EngineeringManager, CodingStandardsEngine};

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

/// Get TARS-enhanced response with personality and engineering focus
pub async fn get_tars_response(source: LlmSource, prompt: &str, context: &str) -> String {
    // Apply TARS personality processing to the prompt
    let enhanced_prompt = TARSCore::process_with_personality(prompt, context).await;
    
    // Get base AI response
    let base_response = get_response(source, &enhanced_prompt).await;
    
    // Apply TARS personality filter to the response
    let personality = crate::personality::TARSPersonality::get_current_state().await;
    let final_response = personality.apply_personality_filter(&base_response, context).await;
    
    final_response
}

/// Conduct code review with TARS engineering manager capabilities
pub async fn conduct_code_review(code: &str, language: &str, context: &str) -> String {
    let engineering_manager = EngineeringManager::new().await;
    let review_result = engineering_manager.conduct_code_review(code, language, context).await;
    
    // Format the review result as a TARS response
    let mut response = review_result.tars_commentary;
    
    if !review_result.violations.is_empty() {
        response.push_str("\n\n[DETAILED ANALYSIS]\n");
        for (i, violation) in review_result.violations.iter().enumerate() {
            response.push_str(&format!("{}. {} (Line: {})\n", 
                i + 1, 
                violation.description,
                violation.line_numbers.first().unwrap_or(&0)
            ));
        }
    }
    
    if !review_result.suggestions.is_empty() {
        response.push_str("\n[RECOMMENDED ACTIONS]\n");
        for (i, suggestion) in review_result.suggestions.iter().enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1, suggestion));
        }
    }
    
    response
}

/// Get coding standards report for a language
pub async fn get_coding_standards_report(language: &str) -> String {
    let standards_engine = CodingStandardsEngine::new().await;
    standards_engine.generate_tars_standards_report(language).await
}

/// Get engineering recommendations for a tech stack
pub async fn get_stack_recommendations(stack: Vec<&str>) -> String {
    let engineering_manager = EngineeringManager::new().await;
    let recommendations = engineering_manager.get_stack_recommendations(&stack).await;
    
    let mut response = String::from("[ENGINEERING RECOMMENDATIONS]\n");
    response.push_str("================================\n\n");
    
    for recommendation in recommendations {
        let priority_marker = match recommendation.priority {
            crate::personality::engineering_manager::RecommendationPriority::Critical => "[CRITICAL]",
            crate::personality::engineering_manager::RecommendationPriority::High => "[HIGH]",
            crate::personality::engineering_manager::RecommendationPriority::Medium => "[MEDIUM]",
            crate::personality::engineering_manager::RecommendationPriority::Low => "[LOW]",
        };
        
        response.push_str(&format!("{} {}: {}\n", 
            priority_marker, 
            recommendation.title,
            recommendation.description
        ));
        response.push_str(&format!("Category: {}\n\n", recommendation.category));
    }
    
    response.push_str("[MISSION PRIORITY] Follow high and critical recommendations for optimal engineering outcomes.");
    response
}

/// Adjust TARS personality settings
pub async fn adjust_tars_personality(humor: Option<f32>, honesty: Option<f32>, sarcasm: Option<f32>) -> Result<String, String> {
    TARSCore::adjust_personality(humor, honesty, sarcasm).await?;
    
    let current_state = TARSCore::get_personality_status().await;
    
    Ok(format!(
        "[PERSONALITY UPDATE COMPLETE]\nHumor: {}%\nHonesty: {}%\nSarcasm: {}%\nMission Focus: 100%\n\nThat's what I would have said. Eventually.",
        (current_state.humor * 100.0) as u8,
        (current_state.honesty * 100.0) as u8,
        (current_state.sarcasm * 100.0) as u8
    ))
}
