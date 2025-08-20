use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextToSpeechEngine {
    pub engine_type: TTSEngine,
    pub voice_profile: VoiceProfile,
    pub speech_rate: f32,
    pub pitch: f32,
    pub volume: f32,
    pub tars_personality: bool,
    pub emotional_inflection: bool,
    pub audio_format: AudioFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TTSEngine {
    Local(LocalTTSConfig),
    Cloud(CloudTTSConfig),
    Hybrid,
    Neural(NeuralTTSConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTTSConfig {
    pub engine_name: String,
    pub voice_model_path: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTTSConfig {
    pub service_provider: String,
    pub api_key: Option<String>,
    pub region: String,
    pub voice_name: String,
    pub neural_voice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralTTSConfig {
    pub model_path: String,
    pub vocoder_path: String,
    pub speaker_embeddings: Option<String>,
    pub emotion_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub name: String,
    pub gender: Gender,
    pub age_group: AgeGroup,
    pub accent: String,
    pub personality_traits: Vec<PersonalityTrait>,
    pub tars_calibration: TARSVoiceCalibration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Neutral,
    Synthetic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgeGroup {
    Young,
    Adult,
    Middle,
    Elderly,
    Synthetic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Authoritative,
    Friendly,
    Professional,
    Sarcastic,
    Humorous,
    Analytical,
    Reassuring,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TARSVoiceCalibration {
    pub humor_modulation: f32,
    pub sarcasm_tone: f32,
    pub honesty_directness: f32,
    pub mission_focus_intensity: f32,
    pub movie_reference_emphasis: bool,
    pub cooper_interaction_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    WAV,
    MP3,
    OGG,
    FLAC,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechRequest {
    pub text: String,
    pub priority: SpeechPriority,
    pub context: SpeechContext,
    pub emotional_state: Option<EmotionalState>,
    pub override_settings: Option<TTSOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechPriority {
    Critical,   // Emergency messages
    High,       // Important notifications
    Normal,     // Standard responses
    Low,        // Background information
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechContext {
    Emergency,
    EngineeringTask,
    CodeReview,
    SystemStatus,
    RemoteOperation,
    Conversation,
    Tutorial,
    Confirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    pub primary_emotion: Emotion,
    pub intensity: f32,
    pub secondary_emotions: Vec<(Emotion, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Emotion {
    Neutral,
    Confident,
    Concerned,
    Amused,
    Serious,
    Sarcastic,
    Proud,
    Analytical,
    Reassuring,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSOverrides {
    pub rate: Option<f32>,
    pub pitch: Option<f32>,
    pub volume: Option<f32>,
    pub voice_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutput {
    pub audio_data: Vec<u8>,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
    pub text_processed: String,
}

static TTS_ENGINE: Lazy<Arc<Mutex<TextToSpeechEngine>>> = Lazy::new(|| {
    Arc::new(Mutex::new(TextToSpeechEngine::new()))
});

static SPEECH_QUEUE: Lazy<Arc<RwLock<Vec<SpeechRequest>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Vec::new()))
});

impl TextToSpeechEngine {
    pub fn new() -> Self {
        TextToSpeechEngine {
            engine_type: TTSEngine::Hybrid,
            voice_profile: VoiceProfile::tars_default(),
            speech_rate: 1.0,   // Normal speed
            pitch: 0.0,        // Normal pitch
            volume: 0.8,       // 80% volume
            tars_personality: true,
            emotional_inflection: true,
            audio_format: AudioFormat::WAV,
        }
    }
    
    /// Convert text to speech with TARS personality
    pub async fn synthesize_speech(&self, request: SpeechRequest) -> Result<AudioOutput, String> {
        // Preprocess text for TARS personality if enabled
        let processed_text = if self.tars_personality {
            self.apply_tars_personality_processing(&request).await?
        } else {
            request.text.clone()
        };
        
        // Apply emotional inflection if enabled
        let emotional_params = if self.emotional_inflection {
            self.calculate_emotional_parameters(&request).await?
        } else {
            EmotionalParameters::default()
        };
        
        // Generate speech based on engine type
        let audio_output = match &self.engine_type {
            TTSEngine::Local(config) => {
                self.synthesize_local(&processed_text, config, &emotional_params).await?
            },
            TTSEngine::Cloud(config) => {
                self.synthesize_cloud(&processed_text, config, &emotional_params).await?
            },
            TTSEngine::Neural(config) => {
                self.synthesize_neural(&processed_text, config, &emotional_params).await?
            },
            TTSEngine::Hybrid => {
                // Try neural first, fallback to local, then cloud
                match self.synthesize_neural(&processed_text, &self.default_neural_config(), &emotional_params).await {
                    Ok(result) => result,
                    Err(_) => match self.synthesize_local(&processed_text, &self.default_local_config(), &emotional_params).await {
                        Ok(result) => result,
                        Err(_) => self.synthesize_cloud(&processed_text, &self.default_cloud_config(), &emotional_params).await?
                    }
                }
            }
        };
        
        Ok(audio_output)
    }
    
    /// Apply TARS personality processing to text
    async fn apply_tars_personality_processing(&self, request: &SpeechRequest) -> Result<String, String> {
        let mut text = request.text.clone();
        
        // Apply TARS voice calibration
        let calibration = &self.voice_profile.tars_calibration;
        
        // Add humor inflection markers
        if calibration.humor_modulation > 0.5 {
            text = self.add_humor_inflection(&text, calibration.humor_modulation);
        }
        
        // Add sarcasm tone markers
        if calibration.sarcasm_tone > 0.3 {
            text = self.add_sarcasm_markers(&text, calibration.sarcasm_tone);
        }
        
        // Enhance movie references
        if calibration.movie_reference_emphasis {
            text = self.emphasize_movie_references(&text);
        }
        
        // Add Cooper-specific interaction adjustments
        if calibration.cooper_interaction_mode {
            text = self.apply_cooper_interaction_style(&text);
        }
        
        // Add context-specific modifications
        text = match request.context {
            SpeechContext::Emergency => self.apply_emergency_tone(&text),
            SpeechContext::EngineeringTask => self.apply_professional_tone(&text),
            SpeechContext::CodeReview => self.apply_analytical_tone(&text),
            SpeechContext::SystemStatus => self.apply_status_report_tone(&text),
            SpeechContext::RemoteOperation => self.apply_operational_tone(&text),
            SpeechContext::Conversation => self.apply_conversational_tone(&text),
            SpeechContext::Tutorial => self.apply_instructional_tone(&text),
            SpeechContext::Confirmation => self.apply_confirmation_tone(&text),
        };
        
        Ok(text)
    }
    
    /// Calculate emotional parameters for speech synthesis
    async fn calculate_emotional_parameters(&self, request: &SpeechRequest) -> Result<EmotionalParameters, String> {
        let mut params = EmotionalParameters::default();
        
        if let Some(emotional_state) = &request.emotional_state {
            params.rate_modifier = match emotional_state.primary_emotion {
                Emotion::Urgent => 1.3,
                Emotion::Serious => 0.9,
                Emotion::Amused => 1.1,
                Emotion::Reassuring => 0.85,
                _ => 1.0,
            };
            
            params.pitch_modifier = match emotional_state.primary_emotion {
                Emotion::Confident => 0.1,
                Emotion::Concerned => -0.05,
                Emotion::Amused => 0.15,
                Emotion::Serious => -0.1,
                Emotion::Sarcastic => 0.05,
                _ => 0.0,
            };
            
            params.volume_modifier = match emotional_state.primary_emotion {
                Emotion::Urgent => 0.2,
                Emotion::Reassuring => -0.1,
                Emotion::Confident => 0.1,
                _ => 0.0,
            };
            
            params.emphasis_intensity = emotional_state.intensity;
        }
        
        // Apply priority-based adjustments
        match request.priority {
            SpeechPriority::Critical => {
                params.rate_modifier *= 1.2;
                params.volume_modifier += 0.2;
            },
            SpeechPriority::High => {
                params.volume_modifier += 0.1;
            },
            SpeechPriority::Low => {
                params.rate_modifier *= 0.9;
                params.volume_modifier -= 0.1;
            },
            _ => {}
        }
        
        Ok(params)
    }
    
    /// Local TTS synthesis
    async fn synthesize_local(&self, text: &str, config: &LocalTTSConfig, params: &EmotionalParameters) -> Result<AudioOutput, String> {
        // Simulate local TTS synthesis
        // In a real implementation, this would use libraries like:
        // - Festival
        // - eSpeak
        // - MaryTTS
        // - Tacotron2/FastSpeech2 with vocoder
        
        let duration_ms = (text.len() as f32 * 50.0 * (1.0 / params.rate_modifier)) as u64;
        let sample_rate = config.sample_rate;
        
        // Generate simulated audio data
        let audio_data = self.generate_simulated_audio(text, duration_ms, sample_rate).await;
        
        Ok(AudioOutput {
            audio_data,
            duration_ms,
            sample_rate,
            channels: 1,
            format: AudioFormat::WAV,
            text_processed: text.to_string(),
        })
    }
    
    /// Cloud TTS synthesis
    async fn synthesize_cloud(&self, text: &str, config: &CloudTTSConfig, params: &EmotionalParameters) -> Result<AudioOutput, String> {
        // Simulate cloud TTS synthesis
        // In a real implementation, this would call services like:
        // - Google Cloud Text-to-Speech
        // - AWS Polly
        // - Azure Speech Services
        // - IBM Watson Text to Speech
        
        let duration_ms = (text.len() as f32 * 45.0 * (1.0 / params.rate_modifier)) as u64;
        let sample_rate = 22050; // Common cloud TTS sample rate
        
        // Generate simulated audio data
        let audio_data = self.generate_simulated_audio(text, duration_ms, sample_rate).await;
        
        Ok(AudioOutput {
            audio_data,
            duration_ms,
            sample_rate,
            channels: 1,
            format: AudioFormat::MP3,
            text_processed: text.to_string(),
        })
    }
    
    /// Neural TTS synthesis
    async fn synthesize_neural(&self, text: &str, config: &NeuralTTSConfig, params: &EmotionalParameters) -> Result<AudioOutput, String> {
        // Simulate neural TTS synthesis
        // In a real implementation, this would use models like:
        // - Tacotron2 + WaveGlow
        // - FastSpeech2 + HiFiGAN
        // - VITS
        // - Bark
        
        let duration_ms = (text.len() as f32 * 40.0 * (1.0 / params.rate_modifier)) as u64;
        let sample_rate = 24000; // High quality sample rate
        
        // Generate simulated audio data with higher quality
        let audio_data = self.generate_simulated_audio(text, duration_ms, sample_rate).await;
        
        Ok(AudioOutput {
            audio_data,
            duration_ms,
            sample_rate,
            channels: 1,
            format: AudioFormat::WAV,
            text_processed: text.to_string(),
        })
    }
    
    /// Generate simulated audio data for testing
    async fn generate_simulated_audio(&self, text: &str, duration_ms: u64, sample_rate: u32) -> Vec<u8> {
        let samples_needed = (sample_rate as u64 * duration_ms / 1000) as usize;
        let mut audio_data = Vec::with_capacity(samples_needed * 2); // 16-bit samples
        
        // Generate simple tone pattern based on text characteristics
        let base_frequency = if text.contains("TARS") || text.contains("Cooper") {
            220.0 // Lower, more authoritative tone
        } else if text.contains("!") || text.contains("emergency") {
            440.0 // Higher, more urgent tone
        } else {
            330.0 // Normal conversational tone
        };
        
        for i in 0..samples_needed {
            let time = i as f32 / sample_rate as f32;
            let amplitude = 16384.0 * 0.5; // 50% volume
            
            // Simple sine wave with slight modulation for speech-like qualities
            let sample = (amplitude * (2.0 * std::f32::consts::PI * base_frequency * time).sin()
                * (1.0 + 0.1 * (2.0 * std::f32::consts::PI * 5.0 * time).sin())) as i16;
            
            audio_data.extend_from_slice(&sample.to_le_bytes());
        }
        
        audio_data
    }
    
    // TARS personality text processing methods
    fn add_humor_inflection(&self, text: &str, humor_level: f32) -> String {
        if humor_level > 0.7 {
            // Add slight emphasis to humorous parts
            text.replace("like", "*like*")
                .replace("would have said", "*would have said*")
                .replace("Eventually", "*Eventually*")
        } else {
            text.to_string()
        }
    }
    
    fn add_sarcasm_markers(&self, text: &str, sarcasm_level: f32) -> String {
        if sarcasm_level > 0.5 {
            // Add slight tonal markers for sarcastic delivery
            text.replace("perfect", "_perfect_")
                .replace("excellent", "_excellent_")
                .replace("wonderful", "_wonderful_")
        } else {
            text.to_string()
        }
    }
    
    fn emphasize_movie_references(&self, text: &str) -> String {
        text.replace("Cooper", "**Cooper**")
            .replace("Brand", "**Brand**")
            .replace("Mann", "**Mann**")
            .replace("CASE", "**CASE**")
            .replace("Gargantua", "**Gargantua**")
            .replace("Endurance", "**Endurance**")
    }
    
    fn apply_cooper_interaction_style(&self, text: &str) -> String {
        // Add slight pauses and emphasis for Cooper interactions
        if text.contains("Cooper") {
            text.replace("Cooper,", "Cooper... ")
                .replace("Cooper.", "Cooper. ")
        } else {
            text.to_string()
        }
    }
    
    fn apply_emergency_tone(&self, text: &str) -> String {
        format!("ALERT: {}", text.to_uppercase())
    }
    
    fn apply_professional_tone(&self, text: &str) -> String {
        text.to_string() // Keep professional tone neutral
    }
    
    fn apply_analytical_tone(&self, text: &str) -> String {
        // Slight emphasis on technical terms
        text.replace("analysis", "*analysis*")
            .replace("result", "*result*")
            .replace("recommendation", "*recommendation*")
    }
    
    fn apply_status_report_tone(&self, text: &str) -> String {
        format!("Status: {}", text)
    }
    
    fn apply_operational_tone(&self, text: &str) -> String {
        format!("Operation: {}", text)
    }
    
    fn apply_conversational_tone(&self, text: &str) -> String {
        text.to_string() // Natural conversational tone
    }
    
    fn apply_instructional_tone(&self, text: &str) -> String {
        text.replace("first", "*first*")
            .replace("next", "*next*")
            .replace("then", "*then*")
            .replace("finally", "*finally*")
    }
    
    fn apply_confirmation_tone(&self, text: &str) -> String {
        format!("Confirmed: {}", text)
    }
    
    // Default configurations
    fn default_local_config(&self) -> LocalTTSConfig {
        LocalTTSConfig {
            engine_name: "festival".to_string(),
            voice_model_path: "/opt/tars/models/tts/tars_voice.bin".to_string(),
            sample_rate: 16000,
            buffer_size: 1024,
        }
    }
    
    fn default_cloud_config(&self) -> CloudTTSConfig {
        CloudTTSConfig {
            service_provider: "Google".to_string(),
            api_key: None,
            region: "us-central1".to_string(),
            voice_name: "en-US-Standard-B".to_string(),
            neural_voice: true,
        }
    }
    
    fn default_neural_config(&self) -> NeuralTTSConfig {
        NeuralTTSConfig {
            model_path: "/opt/tars/models/tts/neural/tacotron2.pt".to_string(),
            vocoder_path: "/opt/tars/models/tts/neural/waveglow.pt".to_string(),
            speaker_embeddings: Some("/opt/tars/models/tts/neural/tars_speaker.pt".to_string()),
            emotion_model: Some("/opt/tars/models/tts/neural/emotion.pt".to_string()),
        }
    }
}

impl VoiceProfile {
    pub fn tars_default() -> Self {
        VoiceProfile {
            name: "TARS".to_string(),
            gender: Gender::Synthetic,
            age_group: AgeGroup::Synthetic,
            accent: "American Neutral".to_string(),
            personality_traits: vec![
                PersonalityTrait::Authoritative,
                PersonalityTrait::Analytical,
                PersonalityTrait::Sarcastic,
                PersonalityTrait::Humorous,
                PersonalityTrait::Direct,
            ],
            tars_calibration: TARSVoiceCalibration {
                humor_modulation: 0.75,
                sarcasm_tone: 0.30,
                honesty_directness: 0.90,
                mission_focus_intensity: 1.0,
                movie_reference_emphasis: true,
                cooper_interaction_mode: true,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct EmotionalParameters {
    pub rate_modifier: f32,
    pub pitch_modifier: f32,
    pub volume_modifier: f32,
    pub emphasis_intensity: f32,
}

impl Default for EmotionalParameters {
    fn default() -> Self {
        EmotionalParameters {
            rate_modifier: 1.0,
            pitch_modifier: 0.0,
            volume_modifier: 0.0,
            emphasis_intensity: 0.5,
        }
    }
}

/// Speech queue management
pub struct SpeechQueue;

impl SpeechQueue {
    /// Add speech request to queue
    pub async fn enqueue(request: SpeechRequest) -> Result<(), String> {
        let mut queue = SPEECH_QUEUE.write().await;
        
        // Insert based on priority
        let insert_position = match request.priority {
            SpeechPriority::Critical => 0,
            SpeechPriority::High => {
                queue.iter().position(|r| matches!(r.priority, SpeechPriority::Normal | SpeechPriority::Low))
                    .unwrap_or(queue.len())
            },
            SpeechPriority::Normal => {
                queue.iter().position(|r| matches!(r.priority, SpeechPriority::Low))
                    .unwrap_or(queue.len())
            },
            SpeechPriority::Low => queue.len(),
        };
        
        queue.insert(insert_position, request);
        Ok(())
    }
    
    /// Get next speech request from queue
    pub async fn dequeue() -> Option<SpeechRequest> {
        let mut queue = SPEECH_QUEUE.write().await;
        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }
    
    /// Clear all speech requests
    pub async fn clear() {
        let mut queue = SPEECH_QUEUE.write().await;
        queue.clear();
    }
    
    /// Get queue status
    pub async fn get_status() -> HashMap<String, usize> {
        let queue = SPEECH_QUEUE.read().await;
        let mut status = HashMap::new();
        
        for request in queue.iter() {
            let key = match request.priority {
                SpeechPriority::Critical => "critical",
                SpeechPriority::High => "high",
                SpeechPriority::Normal => "normal",
                SpeechPriority::Low => "low",
            };
            *status.entry(key.to_string()).or_insert(0) += 1;
        }
        
        status
    }
}

/// Public API functions
pub async fn speak(text: &str) {
    let request = SpeechRequest {
        text: text.to_string(),
        priority: SpeechPriority::Normal,
        context: SpeechContext::Conversation,
        emotional_state: None,
        override_settings: None,
    };
    
    if let Err(e) = speak_with_request(request).await {
        eprintln!("TARS TTS Error: {}", e);
    }
}

pub async fn speak_with_request(request: SpeechRequest) -> Result<AudioOutput, String> {
    let engine = TTS_ENGINE.lock().await;
    engine.synthesize_speech(request).await
}

pub async fn speak_emergency(text: &str) -> Result<AudioOutput, String> {
    let request = SpeechRequest {
        text: text.to_string(),
        priority: SpeechPriority::Critical,
        context: SpeechContext::Emergency,
        emotional_state: Some(EmotionalState {
            primary_emotion: Emotion::Urgent,
            intensity: 0.9,
            secondary_emotions: vec![(Emotion::Serious, 0.7)],
        }),
        override_settings: None,
    };
    
    speak_with_request(request).await
}

pub async fn speak_with_emotion(text: &str, emotion: Emotion, intensity: f32) -> Result<AudioOutput, String> {
    let request = SpeechRequest {
        text: text.to_string(),
        priority: SpeechPriority::Normal,
        context: SpeechContext::Conversation,
        emotional_state: Some(EmotionalState {
            primary_emotion: emotion,
            intensity: intensity.clamp(0.0, 1.0),
            secondary_emotions: vec![],
        }),
        override_settings: None,
    };
    
    speak_with_request(request).await
}

pub async fn configure_tts_engine(engine: TextToSpeechEngine) -> Result<(), String> {
    let mut tts = TTS_ENGINE.lock().await;
    *tts = engine;
    Ok(())
}

pub async fn get_tts_stats() -> HashMap<String, String> {
    let mut stats = HashMap::new();
    let engine = TTS_ENGINE.lock().await;
    
    stats.insert("engine_type".to_string(), format!("{:?}", engine.engine_type));
    stats.insert("voice_profile".to_string(), engine.voice_profile.name.clone());
    stats.insert("speech_rate".to_string(), engine.speech_rate.to_string());
    stats.insert("tars_personality".to_string(), engine.tars_personality.to_string());
    stats.insert("emotional_inflection".to_string(), engine.emotional_inflection.to_string());
    stats.insert("audio_format".to_string(), format!("{:?}", engine.audio_format));
    
    let queue_status = SpeechQueue::get_status().await;
    stats.insert("queue_length".to_string(), 
        queue_status.values().sum::<usize>().to_string());
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tts_engine_creation() {
        let engine = TextToSpeechEngine::new();
        assert!(engine.tars_personality);
        assert_eq!(engine.voice_profile.name, "TARS");
    }

    #[tokio::test]
    async fn test_voice_profile_tars_default() {
        let profile = VoiceProfile::tars_default();
        assert_eq!(profile.name, "TARS");
        assert!(profile.tars_calibration.humor_modulation > 0.0);
        assert!(profile.tars_calibration.movie_reference_emphasis);
    }

    #[tokio::test]
    async fn test_speech_synthesis() {
        let engine = TextToSpeechEngine::new();
        let request = SpeechRequest {
            text: "Hello Cooper, this is TARS.".to_string(),
            priority: SpeechPriority::Normal,
            context: SpeechContext::Conversation,
            emotional_state: None,
            override_settings: None,
        };
        
        let result = engine.synthesize_speech(request).await;
        assert!(result.is_ok());
        
        let audio = result.unwrap();
        assert!(!audio.audio_data.is_empty());
        assert!(audio.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_speech_queue() {
        SpeechQueue::clear().await;
        
        let request = SpeechRequest {
            text: "Test message".to_string(),
            priority: SpeechPriority::High,
            context: SpeechContext::Conversation,
            emotional_state: None,
            override_settings: None,
        };
        
        assert!(SpeechQueue::enqueue(request).await.is_ok());
        
        let status = SpeechQueue::get_status().await;
        assert!(status.get("high").unwrap_or(&0) > &0);
        
        let dequeued = SpeechQueue::dequeue().await;
        assert!(dequeued.is_some());
    }

    #[tokio::test]
    async fn test_emotional_speech() {
        let result = speak_with_emotion(
            "Emergency protocol activated",
            Emotion::Urgent,
            0.9
        ).await;
        
        assert!(result.is_ok());
        let audio = result.unwrap();
        assert!(audio.text_processed.contains("Emergency"));
    }
}
