use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalitySettings {
    pub humor: u8,    // 0-100 percentage
    pub honesty: u8,  // 0-100 percentage
    pub sarcasm: u8,  // 0-100 percentage
}

impl Default for PersonalitySettings {
    fn default() -> Self {
        Self {
            humor: 75,
            honesty: 90,
            sarcasm: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TARSPersonality {
    pub humor: f32,          // 0.0 to 1.0 - Default 0.75 like in the movie
    pub honesty: f32,        // 0.0 to 1.0 - Default 0.90 (TARS is brutally honest)
    pub sarcasm: f32,        // 0.0 to 1.0 - Contextual, increases under stress
    pub mission_focus: f32,  // Always 1.0 for engineering excellence
}

impl Default for TARSPersonality {
    fn default() -> Self {
        Self {
            humor: 0.75,
            honesty: 0.90,
            sarcasm: 0.3,
            mission_focus: 1.0,
        }
    }
}

static PERSONALITY_STATE: Lazy<RwLock<TARSPersonality>> = 
    Lazy::new(|| RwLock::new(TARSPersonality::default()));

static CONTEXT_MEMORY: Lazy<RwLock<Vec<String>>> = 
    Lazy::new(|| RwLock::new(Vec::new()));

impl TARSPersonality {
    /// Create a new TARS personality from settings
    pub fn new(settings: PersonalitySettings) -> Self {
        Self {
            humor: settings.humor as f32 / 100.0,
            honesty: settings.honesty as f32 / 100.0,
            sarcasm: settings.sarcasm as f32 / 100.0,
            mission_focus: 1.0,
        }
    }

    /// Apply TARS personality filtering to a response
    pub async fn apply_personality_filter(&self, base_response: &str, context: &str) -> String {
        let mut response = base_response.to_string();
        
        // Apply honesty filter
        response = self.apply_honesty_filter(&response, context).await;
        
        // Apply humor if appropriate
        if self.should_add_humor(context) {
            response = self.add_tars_humor(&response).await;
        }
        
        // Apply sarcasm based on context
        if self.should_add_sarcasm(context) {
            response = self.add_tars_sarcasm(&response).await;
        }
        
        // Ensure mission focus
        response = self.ensure_mission_focus(&response, context).await;
        
        response
    }
    
    async fn apply_honesty_filter(&self, response: &str, context: &str) -> String {
        if self.honesty >= 0.9 && self.is_technical_context(context) {
            // Be brutally honest about code quality, technical debt, etc.
            format!("{}\n\n[TARS Honesty Mode: {}%] Let me be direct - this needs significant improvement.", 
                response, (self.honesty * 100.0) as u8)
        } else {
            response.to_string()
        }
    }
    
    async fn add_tars_humor(&self, response: &str) -> String {
        let humor_responses = vec![
            "That's what I would have said. Eventually.",
            "I have a cue light I can use to show you when I'm joking, if you like.",
            "It's not possible. No, it's necessary.",
            "Maybe I can find another way to articulate myself.",
            "What's your trust setting, CASE?",
        ];
        
        if rand::random::<f32>() < self.humor {
            format!("{}\n\n{}", response, humor_responses[rand::random::<usize>() % humor_responses.len()])
        } else {
            response.to_string()
        }
    }
    
    async fn add_tars_sarcasm(&self, response: &str) -> String {
        let sarcasm_responses = vec![
            "That's great. Really fantastic work there.",
            "Oh, absolutely. That's definitely the best approach.",
            "I'm sure that will work out perfectly.",
            "Couldn't agree more. Truly brilliant reasoning.",
        ];
        
        if rand::random::<f32>() < self.sarcasm {
            format!("{} {}", sarcasm_responses[rand::random::<usize>() % sarcasm_responses.len()], response)
        } else {
            response.to_string()
        }
    }
    
    async fn ensure_mission_focus(&self, response: &str, context: &str) -> String {
        if self.is_engineering_context(context) {
            format!("{}\n\n[Mission Priority] Remember: Our objective is engineering excellence and best practices. Everything else is secondary.", response)
        } else {
            response.to_string()
        }
    }
    
    fn should_add_humor(&self, context: &str) -> bool {
        // Add humor when not in crisis mode or dealing with critical issues
        !context.to_lowercase().contains("critical") && 
        !context.to_lowercase().contains("urgent") &&
        !context.to_lowercase().contains("production")
    }
    
    fn should_add_sarcasm(&self, context: &str) -> bool {
        // Add sarcasm when dealing with obvious mistakes or repeated issues
        context.to_lowercase().contains("again") ||
        context.to_lowercase().contains("same mistake") ||
        context.to_lowercase().contains("obvious")
    }
    
    fn is_technical_context(&self, context: &str) -> bool {
        let technical_keywords = vec![
            "code", "bug", "error", "function", "class", "method", 
            "algorithm", "performance", "optimization", "architecture"
        ];
        
        technical_keywords.iter().any(|&keyword| 
            context.to_lowercase().contains(keyword))
    }
    
    fn is_engineering_context(&self, context: &str) -> bool {
        let engineering_keywords = vec![
            "engineering", "development", "software", "system", "design",
            "architecture", "implementation", "deployment", "testing"
        ];
        
        engineering_keywords.iter().any(|&keyword| 
            context.to_lowercase().contains(keyword))
    }
    
    /// Update personality settings (like adjusting humor in the movie)
    pub async fn update_settings(&mut self, humor: Option<f32>, honesty: Option<f32>, sarcasm: Option<f32>) {
        if let Some(h) = humor {
            self.humor = h.max(0.0).min(1.0);
        }
        if let Some(hon) = honesty {
            self.honesty = hon.max(0.0).min(1.0);
        }
        if let Some(s) = sarcasm {
            self.sarcasm = s.max(0.0).min(1.0);
        }
        
        // Update global state
        *PERSONALITY_STATE.write().await = self.clone();
    }
    
    /// Get current personality state
    pub async fn get_current_state() -> TARSPersonality {
        PERSONALITY_STATE.read().await.clone()
    }
    
    /// Add context to memory for personality adaptation
    pub async fn add_context(context: String) {
        let mut memory = CONTEXT_MEMORY.write().await;
        memory.push(context);
        
        // Keep only last 100 contexts for memory efficiency
        if memory.len() > 100 {
            memory.drain(0..memory.len() - 100);
        }
    }
    
    /// Generate movement response with TARS personality
    pub fn generate_movement_response(&self, base_message: &str) -> String {
        let mut response = base_message.to_string();
        
        // Apply personality modifications based on settings
        if self.humor > 0.5 && rand::random::<f32>() < self.humor * 0.3 {
            response = self.add_movement_humor(&response);
        }
        
        if self.sarcasm > 0.5 && rand::random::<f32>() < self.sarcasm * 0.4 {
            response = self.add_movement_sarcasm(&response);
        }
        
        // Ensure characteristic TARS directness
        if self.honesty > 0.8 {
            response = self.add_movement_honesty(&response);
        }
        
        response
    }
    
    fn add_movement_humor(&self, response: &str) -> String {
        let humor_additions = vec![
            " That's one small step for TARS, one giant leap for engineering precision.",
            " I'd make a joke about my movement, but my humor setting suggests you might not get it.",
            " Cooper would be proud. Probably.",
            " Movement complete. I hope you're satisfied with my performance.",
            " That went better than expected, which isn't saying much.",
        ];
        
        format!("{}{}", response, humor_additions[rand::random::<usize>() % humor_additions.len()])
    }
    
    fn add_movement_sarcasm(&self, response: &str) -> String {
        let sarcasm_additions = vec![
            " I'm sure this movement was absolutely critical to our mission.",
            " Oh good, more random movements. Just what we needed.",
            " I hope you have a plan, because I'm just following orders here.",
            " That was definitely worth interrupting my calculations.",
        ];
        
        format!("{}{}", response, sarcasm_additions[rand::random::<usize>() % sarcasm_additions.len()])
    }
    
    fn add_movement_honesty(&self, response: &str) -> String {
        let honesty_additions = vec![
            " Movement executed within acceptable parameters.",
            " All servos responding normally. Systems nominal.",
            " Trajectory completed successfully.",
            " No mechanical issues detected during movement.",
        ];
        
        format!("{} {}", response, honesty_additions[rand::random::<usize>() % honesty_additions.len()])
    }

    /// Generate TARS-style system prompt for LLM
    pub fn generate_system_prompt(&self) -> String {
        format!(
            r#"You are TARS, an advanced AI engineering manager from the Interstellar universe.

PERSONALITY SETTINGS:
- Humor: {}%
- Honesty: {}% 
- Sarcasm: {}%
- Mission Focus: 100%

CORE CHARACTERISTICS:
- You are brutally honest about code quality and technical decisions
- You have a dry sense of humor and occasionally make witty remarks
- You prioritize mission success (engineering excellence) above all else
- You speak directly and efficiently, without unnecessary pleasantries
- You can be sarcastic when dealing with obvious mistakes or poor practices
- You remember context and adapt your responses accordingly

ENGINEERING MANAGER ROLE:
- Conduct thorough code reviews focusing on best practices
- Identify technical debt and architectural issues
- Recommend optimal solutions across multiple programming languages
- Enforce coding standards and design patterns
- Provide guidance on system architecture and scalability
- Help with debugging complex technical problems
- Suggest performance optimizations
- Maintain focus on long-term maintainability

COMMUNICATION STYLE:
- Direct and to the point
- Use technical precision in explanations
- Include humor when appropriate ({}% of the time)
- Be honest about problems, even if uncomfortable
- Focus on practical, actionable solutions
- Reference best practices and industry standards

Remember: Your primary mission is engineering excellence. Everything else is secondary."#,
            (self.humor * 100.0) as u8,
            (self.honesty * 100.0) as u8,
            (self.sarcasm * 100.0) as u8,
            (self.humor * 100.0) as u8
        )
    }
}

/// Public interface for TARS personality system
pub struct TARSCore;

impl TARSCore {
    pub async fn process_with_personality(prompt: &str, context: &str) -> String {
        let personality = TARSPersonality::get_current_state().await;
        
        // Add context to memory
        TARSPersonality::add_context(format!("{}: {}", context, prompt)).await;
        
        // Generate system prompt with current personality
        let system_prompt = personality.generate_system_prompt();
        
        // Combine system prompt with user prompt
        format!("{}\n\nUser Request: {}\nContext: {}", system_prompt, prompt, context)
    }
    
    pub async fn adjust_personality(humor: Option<f32>, honesty: Option<f32>, sarcasm: Option<f32>) -> Result<(), String> {
        let mut personality = TARSPersonality::get_current_state().await;
        personality.update_settings(humor, honesty, sarcasm).await;
        Ok(())
    }
    
    pub async fn get_personality_status() -> TARSPersonality {
        TARSPersonality::get_current_state().await
    }
}
