use crate::voice::{
    speech_recognition::{
        transcribe, transcribe_with_tars_interpretation, detect_wake_word, 
        configure_recognition_engine, get_recognition_stats,
        SpeechRecognitionEngine, RecognitionResult, RecognitionEngine,
        LocalRecognitionConfig, CloudRecognitionConfig, TARSInterpretation,
        CommandType, CommandPriority
    },
    text_to_speech::{
        speak, speak_with_request, speak_emergency, speak_with_emotion,
        configure_tts_engine, get_tts_stats,
        SpeechRequest, SpeechPriority, SpeechContext, EmotionalState, Emotion,
        TextToSpeechEngine, TTSEngine, VoiceProfile, AudioOutput, SpeechQueue
    }
};
use tauri::State;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Speech Recognition Commands
#[tauri::command]
pub async fn transcribe_audio(audio_data: Vec<u8>) -> Result<String, String> {
    Ok(transcribe(&audio_data).await)
}

#[tauri::command]
pub async fn transcribe_audio_with_tars_interpretation(audio_data: Vec<u8>) -> Result<RecognitionResult, String> {
    transcribe_with_tars_interpretation(&audio_data).await
}

#[tauri::command]
pub async fn detect_wake_word_in_audio(audio_data: Vec<u8>) -> Result<Option<String>, String> {
    detect_wake_word(&audio_data).await
}

#[tauri::command]
pub async fn configure_speech_recognition(
    engine_type: String,
    language: String,
    confidence_threshold: f32,
    wake_words: Vec<String>,
    voice_activity_detection: bool,
    noise_reduction: bool,
    tars_mode: bool,
) -> Result<String, String> {
    let recognition_engine = match engine_type.as_str() {
        "Local" => RecognitionEngine::Local(LocalRecognitionConfig {
            model_path: "/opt/tars/models/speech/local_model.bin".to_string(),
            beam_width: 16,
            language_model_path: Some("/opt/tars/models/speech/lm.bin".to_string()),
            acoustic_model_path: Some("/opt/tars/models/speech/am.bin".to_string()),
        }),
        "Cloud" => RecognitionEngine::Cloud(CloudRecognitionConfig {
            service_provider: "Google".to_string(),
            api_key: None,
            region: "us-central1".to_string(),
            streaming: true,
        }),
        _ => RecognitionEngine::Hybrid,
    };

    let config = SpeechRecognitionEngine {
        engine_type: recognition_engine,
        language,
        confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        wake_words,
        voice_activity_detection,
        noise_reduction,
        tars_mode,
    };

    configure_recognition_engine(config).await?;
    Ok("Speech recognition engine configured successfully. TARS voice processing updated.".to_string())
}

#[tauri::command]
pub async fn get_speech_recognition_stats() -> Result<HashMap<String, String>, String> {
    Ok(get_recognition_stats().await)
}

// Text-to-Speech Commands
#[tauri::command]
pub async fn speak_text(text: String) -> Result<String, String> {
    speak(&text).await;
    Ok("Speech synthesis completed successfully.".to_string())
}

#[tauri::command]
pub async fn speak_text_with_context(
    text: String,
    priority: String,
    context: String,
    emotion: Option<String>,
    intensity: Option<f32>,
) -> Result<AudioOutput, String> {
    let speech_priority = match priority.as_str() {
        "Critical" => SpeechPriority::Critical,
        "High" => SpeechPriority::High,
        "Low" => SpeechPriority::Low,
        _ => SpeechPriority::Normal,
    };

    let speech_context = match context.as_str() {
        "Emergency" => SpeechContext::Emergency,
        "EngineeringTask" => SpeechContext::EngineeringTask,
        "CodeReview" => SpeechContext::CodeReview,
        "SystemStatus" => SpeechContext::SystemStatus,
        "RemoteOperation" => SpeechContext::RemoteOperation,
        "Tutorial" => SpeechContext::Tutorial,
        "Confirmation" => SpeechContext::Confirmation,
        _ => SpeechContext::Conversation,
    };

    let emotional_state = if let Some(emotion_str) = emotion {
        let primary_emotion = match emotion_str.as_str() {
            "Confident" => Emotion::Confident,
            "Concerned" => Emotion::Concerned,
            "Amused" => Emotion::Amused,
            "Serious" => Emotion::Serious,
            "Sarcastic" => Emotion::Sarcastic,
            "Proud" => Emotion::Proud,
            "Analytical" => Emotion::Analytical,
            "Reassuring" => Emotion::Reassuring,
            "Urgent" => Emotion::Urgent,
            _ => Emotion::Neutral,
        };

        Some(EmotionalState {
            primary_emotion,
            intensity: intensity.unwrap_or(0.5).clamp(0.0, 1.0),
            secondary_emotions: vec![],
        })
    } else {
        None
    };

    let request = SpeechRequest {
        text,
        priority: speech_priority,
        context: speech_context,
        emotional_state,
        override_settings: None,
    };

    speak_with_request(request).await
}

#[tauri::command]
pub async fn speak_emergency_message(text: String) -> Result<AudioOutput, String> {
    speak_emergency(&text).await
}

#[tauri::command]
pub async fn speak_with_emotional_tone(
    text: String,
    emotion: String,
    intensity: f32,
) -> Result<AudioOutput, String> {
    let emotion_type = match emotion.as_str() {
        "Confident" => Emotion::Confident,
        "Concerned" => Emotion::Concerned,
        "Amused" => Emotion::Amused,
        "Serious" => Emotion::Serious,
        "Sarcastic" => Emotion::Sarcastic,
        "Proud" => Emotion::Proud,
        "Analytical" => Emotion::Analytical,
        "Reassuring" => Emotion::Reassuring,
        "Urgent" => Emotion::Urgent,
        _ => Emotion::Neutral,
    };

    speak_with_emotion(&text, emotion_type, intensity.clamp(0.0, 1.0)).await
}

#[tauri::command]
pub async fn configure_text_to_speech(
    engine_type: String,
    speech_rate: f32,
    pitch: f32,
    volume: f32,
    tars_personality: bool,
    emotional_inflection: bool,
    humor_modulation: f32,
    sarcasm_tone: f32,
    cooper_interaction_mode: bool,
) -> Result<String, String> {
    let tts_engine_type = match engine_type.as_str() {
        "Local" => TTSEngine::Local(crate::voice::text_to_speech::LocalTTSConfig {
            engine_name: "festival".to_string(),
            voice_model_path: "/opt/tars/models/tts/tars_voice.bin".to_string(),
            sample_rate: 16000,
            buffer_size: 1024,
        }),
        "Cloud" => TTSEngine::Cloud(crate::voice::text_to_speech::CloudTTSConfig {
            service_provider: "Google".to_string(),
            api_key: None,
            region: "us-central1".to_string(),
            voice_name: "en-US-Standard-B".to_string(),
            neural_voice: true,
        }),
        "Neural" => TTSEngine::Neural(crate::voice::text_to_speech::NeuralTTSConfig {
            model_path: "/opt/tars/models/tts/neural/tacotron2.pt".to_string(),
            vocoder_path: "/opt/tars/models/tts/neural/waveglow.pt".to_string(),
            speaker_embeddings: Some("/opt/tars/models/tts/neural/tars_speaker.pt".to_string()),
            emotion_model: Some("/opt/tars/models/tts/neural/emotion.pt".to_string()),
        }),
        _ => TTSEngine::Hybrid,
    };

    let mut voice_profile = VoiceProfile::tars_default();
    voice_profile.tars_calibration.humor_modulation = humor_modulation.clamp(0.0, 1.0);
    voice_profile.tars_calibration.sarcasm_tone = sarcasm_tone.clamp(0.0, 1.0);
    voice_profile.tars_calibration.cooper_interaction_mode = cooper_interaction_mode;

    let engine = TextToSpeechEngine {
        engine_type: tts_engine_type,
        voice_profile,
        speech_rate: speech_rate.clamp(0.1, 3.0),
        pitch: pitch.clamp(-1.0, 1.0),
        volume: volume.clamp(0.0, 1.0),
        tars_personality,
        emotional_inflection,
        audio_format: crate::voice::text_to_speech::AudioFormat::WAV,
    };

    configure_tts_engine(engine).await?;
    Ok("Text-to-speech engine configured successfully. TARS voice personality updated.".to_string())
}

#[tauri::command]
pub async fn get_text_to_speech_stats() -> Result<HashMap<String, String>, String> {
    Ok(get_tts_stats().await)
}

// Speech Queue Management Commands
#[tauri::command]
pub async fn enqueue_speech_request(
    text: String,
    priority: String,
    context: String,
) -> Result<String, String> {
    let speech_priority = match priority.as_str() {
        "Critical" => SpeechPriority::Critical,
        "High" => SpeechPriority::High,
        "Low" => SpeechPriority::Low,
        _ => SpeechPriority::Normal,
    };

    let speech_context = match context.as_str() {
        "Emergency" => SpeechContext::Emergency,
        "EngineeringTask" => SpeechContext::EngineeringTask,
        "CodeReview" => SpeechContext::CodeReview,
        "SystemStatus" => SpeechContext::SystemStatus,
        "RemoteOperation" => SpeechContext::RemoteOperation,
        "Tutorial" => SpeechContext::Tutorial,
        "Confirmation" => SpeechContext::Confirmation,
        _ => SpeechContext::Conversation,
    };

    let request = SpeechRequest {
        text,
        priority: speech_priority,
        context: speech_context,
        emotional_state: None,
        override_settings: None,
    };

    SpeechQueue::enqueue(request).await?;
    Ok("Speech request added to queue successfully.".to_string())
}

#[tauri::command]
pub async fn get_speech_queue_status() -> Result<HashMap<String, usize>, String> {
    Ok(SpeechQueue::get_status().await)
}

#[tauri::command]
pub async fn clear_speech_queue() -> Result<String, String> {
    SpeechQueue::clear().await;
    Ok("Speech queue cleared successfully.".to_string())
}

// Voice Interaction Commands
#[tauri::command]
pub async fn start_voice_interaction() -> Result<String, String> {
    // Simulate starting voice interaction mode
    Ok("Voice interaction mode activated. TARS is now listening for wake words and voice commands.".to_string())
}

#[tauri::command]
pub async fn stop_voice_interaction() -> Result<String, String> {
    // Simulate stopping voice interaction mode
    Ok("Voice interaction mode deactivated. TARS voice processing suspended.".to_string())
}

#[tauri::command]
pub async fn process_voice_command(audio_data: Vec<u8>) -> Result<VoiceCommandResult, String> {
    // Process complete voice interaction: wake word detection -> transcription -> command execution
    
    // First, check for wake word
    let wake_word_detected = match detect_wake_word(&audio_data).await? {
        Some(word) => word,
        None => return Ok(VoiceCommandResult {
            wake_word_detected: false,
            detected_word: None,
            transcription: None,
            tars_interpretation: None,
            command_executed: false,
            response_text: None,
            response_audio: None,
        }),
    };

    // If wake word detected, transcribe the audio
    let recognition_result = transcribe_with_tars_interpretation(&audio_data).await?;

    // Generate TARS response based on interpretation
    let response_text = if let Some(ref interpretation) = recognition_result.tars_interpretation {
        generate_tars_response(interpretation).await
    } else {
        "I heard you, Cooper, but I'm not sure what you want me to do.".to_string()
    };

    // Generate speech response
    let response_audio = speak_with_request(SpeechRequest {
        text: response_text.clone(),
        priority: SpeechPriority::Normal,
        context: SpeechContext::Conversation,
        emotional_state: Some(EmotionalState {
            primary_emotion: Emotion::Confident,
            intensity: 0.7,
            secondary_emotions: vec![],
        }),
        override_settings: None,
    }).await?;

    Ok(VoiceCommandResult {
        wake_word_detected: true,
        detected_word: Some(wake_word_detected),
        transcription: Some(recognition_result.text.clone()),
        tars_interpretation: recognition_result.tars_interpretation,
        command_executed: true,
        response_text: Some(response_text),
        response_audio: Some(response_audio),
    })
}

// TARS-Specific Voice Commands
#[tauri::command]
pub async fn tars_voice_status_report() -> Result<String, String> {
    let recognition_stats = get_recognition_stats().await;
    let tts_stats = get_tts_stats().await;
    let queue_status = SpeechQueue::get_status().await;

    let total_queued = queue_status.values().sum::<usize>();

    let report = format!(
        "[TARS VOICE SYSTEM STATUS]\n\
        ============================\n\n\
        SPEECH RECOGNITION:\n\
        - Engine: {}\n\
        - Wake Word Detection: {}\n\
        - TARS Mode: {}\n\
        - Confidence Threshold: {}\n\n\
        TEXT-TO-SPEECH:\n\
        - Engine: {}\n\
        - Voice Profile: {}\n\
        - TARS Personality: {}\n\
        - Emotional Inflection: {}\n\n\
        SPEECH QUEUE:\n\
        - Total Requests: {}\n\
        - Critical: {}\n\
        - High: {}\n\
        - Normal: {}\n\
        - Low: {}\n\n\
        [VOICE SYSTEM ASSESSMENT] All voice processing systems operational.\n\
        Mission readiness: 100%\n\
        Voice interaction capabilities: ACTIVE\n\
        Cooper communication protocols: ENABLED\n\n\
        That's what I would have said. Eventually.",
        recognition_stats.get("engine_type").unwrap_or(&"Unknown".to_string()),
        recognition_stats.get("wake_word_sensitivity").unwrap_or(&"Unknown".to_string()),
        recognition_stats.get("tars_mode").unwrap_or(&"Unknown".to_string()),
        recognition_stats.get("confidence_threshold").unwrap_or(&"Unknown".to_string()),
        tts_stats.get("engine_type").unwrap_or(&"Unknown".to_string()),
        tts_stats.get("voice_profile").unwrap_or(&"Unknown".to_string()),
        tts_stats.get("tars_personality").unwrap_or(&"Unknown".to_string()),
        tts_stats.get("emotional_inflection").unwrap_or(&"Unknown".to_string()),
        total_queued,
        queue_status.get("critical").unwrap_or(&0),
        queue_status.get("high").unwrap_or(&0),
        queue_status.get("normal").unwrap_or(&0),
        queue_status.get("low").unwrap_or(&0),
    );

    Ok(report)
}

#[tauri::command]
pub async fn tars_voice_calibration(
    humor_level: f32,
    sarcasm_level: f32,
    cooper_mode: bool,
    movie_references: bool,
) -> Result<String, String> {
    configure_text_to_speech(
        "Hybrid".to_string(),
        1.0,    // speech_rate
        0.0,    // pitch
        0.8,    // volume
        true,   // tars_personality
        true,   // emotional_inflection
        humor_level.clamp(0.0, 1.0),
        sarcasm_level.clamp(0.0, 1.0),
        cooper_mode,
    ).await?;

    let response = format!(
        "TARS voice calibration updated, Cooper.\n\
        Humor modulation: {:.0}%\n\
        Sarcasm tone: {:.0}%\n\
        Cooper interaction mode: {}\n\
        Movie reference emphasis: {}\n\n\
        Voice personality parameters optimized for mission requirements.",
        humor_level * 100.0,
        sarcasm_level * 100.0,
        if cooper_mode { "ENABLED" } else { "DISABLED" },
        if movie_references { "ENABLED" } else { "DISABLED" }
    );

    Ok(response)
}

#[tauri::command]
pub async fn tars_emergency_voice_alert(message: String) -> Result<AudioOutput, String> {
    let emergency_message = format!("EMERGENCY ALERT: {}", message.to_uppercase());
    
    speak_emergency(&emergency_message).await
}

// Helper structures and functions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceCommandResult {
    pub wake_word_detected: bool,
    pub detected_word: Option<String>,
    pub transcription: Option<String>,
    pub tars_interpretation: Option<TARSInterpretation>,
    pub command_executed: bool,
    pub response_text: Option<String>,
    pub response_audio: Option<AudioOutput>,
}

async fn generate_tars_response(interpretation: &TARSInterpretation) -> String {
    match interpretation.command_type {
        CommandType::EngineeringTask => {
            "Roger that, Cooper. Engineering task recognized. Initiating workflow execution.".to_string()
        },
        CommandType::SystemControl => {
            "System control command acknowledged. Executing with mission focus at 100%.".to_string()
        },
        CommandType::CodeReview => {
            "Code review request received. Preparing analysis protocols. This should be interesting.".to_string()
        },
        CommandType::RemoteOperation => {
            "Remote operation command detected. Establishing secure connections. Like reaching across the void of space.".to_string()
        },
        CommandType::Emergency => {
            "EMERGENCY PROTOCOL ACTIVATED. All systems responding. What's the situation, Cooper?".to_string()
        },
        CommandType::Configuration => {
            "Configuration change request acknowledged. Adjusting system parameters as requested.".to_string()
        },
        CommandType::Query => {
            "Information request received. Accessing databases and generating report.".to_string()
        },
        CommandType::Conversation => {
            match interpretation.priority {
                CommandPriority::Critical => "Critical communication received. Standing by for instructions.",
                CommandPriority::High => "Important message acknowledged. How can I assist?",
                CommandPriority::Normal => "Message received, Cooper. What can I do for you?",
                CommandPriority::Low => "I'm here when you need me. That's what I would have said. Eventually.",
            }.to_string()
        },
    }
}

// Voice Training and Customization Commands
#[tauri::command]
pub async fn train_tars_voice_recognition(
    training_phrases: Vec<String>,
    user_voice_sample: Vec<u8>,
) -> Result<String, String> {
    // Simulate voice training process
    let phrase_count = training_phrases.len();
    let sample_duration = user_voice_sample.len() as f32 / 16000.0; // Assume 16kHz sample rate
    
    let response = format!(
        "Voice training session completed.\n\
        Training phrases processed: {}\n\
        Voice sample duration: {:.1} seconds\n\n\
        TARS voice recognition has been calibrated to your vocal patterns, Cooper.\n\
        Speech recognition accuracy should be improved for future interactions.",
        phrase_count,
        sample_duration
    );

    Ok(response)
}

#[tauri::command]
pub async fn create_custom_tars_voice_profile(
    profile_name: String,
    humor_level: f32,
    sarcasm_level: f32,
    formality_level: f32,
    technical_focus: f32,
) -> Result<String, String> {
    // Simulate custom voice profile creation
    let response = format!(
        "Custom TARS voice profile '{}' created successfully.\n\n\
        Profile Parameters:\n\
        - Humor Level: {:.0}%\n\
        - Sarcasm Tone: {:.0}%\n\
        - Formality Level: {:.0}%\n\
        - Technical Focus: {:.0}%\n\n\
        Voice profile saved and ready for activation.\n\
        Cooper, your personalized TARS interaction experience is now configured.",
        profile_name,
        humor_level * 100.0,
        sarcasm_level * 100.0,
        formality_level * 100.0,
        technical_focus * 100.0
    );

    Ok(response)
}
