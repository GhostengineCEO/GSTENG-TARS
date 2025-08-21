use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;
use crate::personality::tars_core::TARSPersonality;
use super::advanced_tts::{SynthesisConfig, EmotionConfig, AudioFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TARSVoiceProfile {
    pub profile_name: String,
    pub acoustic_parameters: AcousticParameters,
    pub speech_patterns: SpeechPatterns,
    pub voice_effects: VoiceEffects,
    pub emotional_range: EmotionalRange,
    pub movie_accuracy_level: f32, // 0.0-1.0, how close to original TARS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticParameters {
    pub fundamental_frequency: FrequencyRange,
    pub formant_frequencies: FormantStructure,
    pub harmonic_content: HarmonicProfile,
    pub vocal_tract_length: f32, // Simulated vocal tract length in cm
    pub voice_quality_factors: VoiceQualityFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyRange {
    pub base_f0: f32,        // Base fundamental frequency (Hz)
    pub min_f0: f32,         // Minimum F0 for emphasis (Hz)
    pub max_f0: f32,         // Maximum F0 for emphasis (Hz)
    pub typical_variation: f32, // Typical F0 variation (Hz)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormantStructure {
    pub f1: f32,  // First formant (Hz) - vowel height
    pub f2: f32,  // Second formant (Hz) - vowel frontness
    pub f3: f32,  // Third formant (Hz) - rhoticity/liquids
    pub f4: f32,  // Fourth formant (Hz) - voice quality
    pub f5: f32,  // Fifth formant (Hz) - voice quality
    pub bandwidth_factors: [f32; 5], // Formant bandwidths
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicProfile {
    pub fundamental_amplitude: f32,    // Relative amplitude of F0
    pub second_harmonic: f32,         // Relative amplitude of 2*F0
    pub third_harmonic: f32,          // Relative amplitude of 3*F0
    pub fourth_harmonic: f32,         // Relative amplitude of 4*F0
    pub harmonic_decay_rate: f32,     // How quickly harmonics decay
    pub noise_floor: f32,             // Background noise level
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceQualityFactors {
    pub breathiness: f32,      // 0.0-1.0, amount of breath noise
    pub roughness: f32,        // 0.0-1.0, vocal fold irregularity
    pub tenseness: f32,        // 0.0-1.0, vocal fold tension
    pub resonance: f32,        // 0.0-1.0, oral/nasal resonance balance
    pub metallic_quality: f32, // 0.0-1.0, synthetic/robotic quality
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechPatterns {
    pub speaking_rate: SpeakingRateProfile,
    pub rhythm_patterns: RhythmPatterns,
    pub pause_patterns: PausePatterns,
    pub stress_patterns: StressPatterns,
    pub articulation_precision: f32, // 0.0-1.0, how precisely articulated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingRateProfile {
    pub words_per_minute: f32,        // Base speaking rate
    pub syllables_per_second: f32,    // Syllable rate
    pub contextual_variations: HashMap<String, f32>, // Rate changes by context
    pub emphasis_slowdown: f32,       // How much to slow for emphasis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmPatterns {
    pub stress_timing: bool,          // Stress-timed vs syllable-timed
    pub isochrony_strength: f32,      // 0.0-1.0, rhythmic regularity
    pub foot_structure: String,       // Metrical foot pattern
    pub sentence_rhythm: SentenceRhythm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceRhythm {
    pub initial_stress: f32,          // Emphasis on sentence beginning
    pub final_lowering: f32,          // F0 drop at sentence end
    pub declarative_contour: Vec<f32>, // F0 contour for statements
    pub interrogative_contour: Vec<f32>, // F0 contour for questions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausePatterns {
    pub comma_pause_ms: u32,          // Pause duration for commas
    pub period_pause_ms: u32,         // Pause duration for periods
    pub breath_pause_ms: u32,         // Natural breath pauses
    pub emphasis_pause_ms: u32,       // Dramatic pauses for emphasis
    pub hesitation_patterns: Vec<String>, // Um, ah, etc. (TARS doesn't use these)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressPatterns {
    pub lexical_stress: LexicalStressProfile,
    pub sentence_stress: SentenceStressProfile,
    pub contrastive_stress: ContrastiveStressProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalStressProfile {
    pub primary_stress_f0_boost: f32,    // Hz increase for primary stress
    pub secondary_stress_f0_boost: f32,  // Hz increase for secondary stress
    pub stress_duration_factor: f32,     // Duration multiplier for stressed syllables
    pub stress_amplitude_boost: f32,     // Amplitude increase for stress
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceStressProfile {
    pub focus_stress_boost: f32,         // Extra emphasis for focused words
    pub new_information_stress: f32,     // Stress for new information
    pub contrast_stress_boost: f32,      // Stress for contrasted elements
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastiveStressProfile {
    pub contrastive_f0_range: f32,       // F0 range expansion for contrast
    pub contrastive_duration: f32,       // Duration change for contrast
    pub contrastive_pause_before: u32,   // Pause before contrastive element
    pub contrastive_pause_after: u32,    // Pause after contrastive element
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceEffects {
    pub robotic_filter: RoboticFilter,
    pub reverb_settings: ReverbSettings,
    pub equalization: EqualizationCurve,
    pub dynamic_processing: DynamicProcessing,
    pub servo_sounds: ServoSounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoboticFilter {
    pub enabled: bool,
    pub metallic_resonance: f32,         // Amount of metallic coloration
    pub synthetic_harmonics: f32,        // Addition of synthetic harmonics
    pub formant_shift: f32,             // Shift formants for robotic quality
    pub spectral_tilt: f32,             // High-frequency emphasis/de-emphasis
    pub quantization_noise: f32,        // Subtle digital artifacts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbSettings {
    pub room_size: f32,                 // Simulated room size (0.0-1.0)
    pub decay_time: f32,                // Reverb decay time (seconds)
    pub early_reflections: f32,         // Early reflection level
    pub diffusion: f32,                 // Reverb diffusion amount
    pub high_frequency_damping: f32,    // HF damping in reverb
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizationCurve {
    pub low_shelf: EQBand,              // Low-frequency shelf
    pub low_mid: EQBand,                // Low-mid parametric
    pub mid: EQBand,                    // Mid-range parametric
    pub high_mid: EQBand,               // High-mid parametric
    pub high_shelf: EQBand,             // High-frequency shelf
    pub presence_boost: f32,            // Presence range boost for clarity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EQBand {
    pub frequency: f32,                 // Center frequency (Hz)
    pub gain: f32,                      // Gain in dB
    pub q_factor: f32,                  // Q factor (bandwidth)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicProcessing {
    pub compressor: CompressorSettings,
    pub limiter: LimiterSettings,
    pub noise_gate: NoiseGateSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressorSettings {
    pub threshold: f32,                 // Compression threshold (dB)
    pub ratio: f32,                     // Compression ratio
    pub attack_time: f32,               // Attack time (ms)
    pub release_time: f32,              // Release time (ms)
    pub knee: f32,                      // Soft/hard knee
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimiterSettings {
    pub threshold: f32,                 // Limiting threshold (dB)
    pub release_time: f32,              // Release time (ms)
    pub lookahead: f32,                 // Lookahead time (ms)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseGateSettings {
    pub threshold: f32,                 // Gate threshold (dB)
    pub attack_time: f32,               // Gate attack (ms)
    pub hold_time: f32,                 // Gate hold (ms)
    pub release_time: f32,              // Gate release (ms)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoSounds {
    pub enabled: bool,
    pub servo_frequency: f32,           // Base frequency of servo sounds (Hz)
    pub servo_amplitude: f32,           // Relative amplitude of servo sounds
    pub between_sentences: bool,        // Add servo sounds between sentences
    pub during_pauses: bool,            // Add subtle servo sounds during pauses
    pub movement_correlation: f32,      // Correlation with physical movement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalRange {
    pub base_emotions: HashMap<String, EmotionalVoiceState>,
    pub tars_specific_states: TARSEmotionalStates,
    pub transition_smoothing: f32,      // How smoothly to transition between emotions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalVoiceState {
    pub f0_modifier: f32,               // Fundamental frequency change
    pub formant_shifts: [f32; 5],       // Formant frequency shifts
    pub speaking_rate_modifier: f32,    // Rate change for this emotion
    pub amplitude_modifier: f32,        // Volume change for this emotion
    pub voice_quality_changes: VoiceQualityFactors,
    pub articulation_changes: f32,      // Precision change
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TARSEmotionalStates {
    pub deadpan_humor: EmotionalVoiceState,
    pub mission_focused: EmotionalVoiceState,
    pub emergency_alert: EmotionalVoiceState,
    pub analytical_mode: EmotionalVoiceState,
    pub cooper_interaction: EmotionalVoiceState,
    pub sarcastic_response: EmotionalVoiceState,
    pub reassuring_tone: EmotionalVoiceState,
    pub status_report: EmotionalVoiceState,
}

impl TARSVoiceProfile {
    /// Create the movie-accurate TARS voice profile
    pub fn interstellar_accurate() -> Self {
        TARSVoiceProfile {
            profile_name: "TARS_Interstellar_Movie".to_string(),
            acoustic_parameters: AcousticParameters::tars_movie_accurate(),
            speech_patterns: SpeechPatterns::tars_speech_patterns(),
            voice_effects: VoiceEffects::tars_voice_effects(),
            emotional_range: EmotionalRange::tars_emotional_range(),
            movie_accuracy_level: 0.95, // 95% movie accuracy target
        }
    }

    /// Create a more synthetic TARS variant
    pub fn synthetic_tars() -> Self {
        let mut profile = Self::interstellar_accurate();
        profile.profile_name = "TARS_Synthetic".to_string();
        profile.voice_effects.robotic_filter.metallic_resonance = 0.6;
        profile.voice_effects.robotic_filter.synthetic_harmonics = 0.4;
        profile.movie_accuracy_level = 0.7;
        profile
    }

    /// Create synthesis config from voice profile
    pub fn to_synthesis_config(&self, text: &str, context: &str) -> SynthesisConfig {
        let mut config = SynthesisConfig::default();
        
        // Set base parameters
        config.voice_profile = self.profile_name.clone();
        config.speaking_rate = self.speech_patterns.speaking_rate.words_per_minute / 150.0; // Normalize to 150 WPM baseline
        config.pitch = (self.acoustic_parameters.fundamental_frequency.base_f0 - 220.0) / 50.0; // Normalize
        config.volume = 0.8; // TARS speaks at consistent volume
        
        // Apply contextual modifications
        if let Some(context_rate) = self.speech_patterns.speaking_rate.contextual_variations.get(context) {
            config.speaking_rate *= context_rate;
        }
        
        // Set emotion based on text analysis
        config.emotion = self.analyze_text_for_emotion(text, context);
        
        config.sample_rate = 24000; // High quality for TARS
        config.bit_depth = 16;
        config.output_format = AudioFormat::WAV;
        
        config
    }

    /// Analyze text to determine appropriate emotional state
    fn analyze_text_for_emotion(&self, text: &str, context: &str) -> Option<EmotionConfig> {
        let text_lower = text.to_lowercase();
        
        // Emergency detection
        if text_lower.contains("emergency") || text_lower.contains("alert") || text_lower.contains("danger") {
            return Some(EmotionConfig {
                primary_emotion: "emergency_alert".to_string(),
                intensity: 0.9,
                arousal: 0.8,
                valence: -0.2, // Slight negative valence for urgency
            });
        }
        
        // Humor detection
        if text_lower.contains("joke") || text_lower.contains("humor") || text_lower.contains("funny") {
            return Some(EmotionConfig {
                primary_emotion: "deadpan_humor".to_string(),
                intensity: 0.7,
                arousal: 0.3, // Low arousal for deadpan delivery
                valence: 0.5,
            });
        }
        
        // Cooper interaction
        if text_lower.contains("cooper") {
            return Some(EmotionConfig {
                primary_emotion: "cooper_interaction".to_string(),
                intensity: 0.6,
                arousal: 0.4,
                valence: 0.1, // Slight positive for familiarity
            });
        }
        
        // Sarcasm detection
        if text_lower.contains("perfect") || text_lower.contains("excellent") || text_lower.contains("wonderful") {
            return Some(EmotionConfig {
                primary_emotion: "sarcastic_response".to_string(),
                intensity: 0.5,
                arousal: 0.3,
                valence: -0.1, // Slight negative for sarcasm
            });
        }
        
        // Status report
        if context == "status" || text_lower.contains("status") || text_lower.contains("report") {
            return Some(EmotionConfig {
                primary_emotion: "status_report".to_string(),
                intensity: 0.4,
                arousal: 0.2, // Very low arousal for factual reporting
                valence: 0.0,
            });
        }
        
        // Default to mission-focused
        Some(EmotionConfig {
            primary_emotion: "mission_focused".to_string(),
            intensity: 0.5,
            arousal: 0.3,
            valence: 0.0,
        })
    }

    /// Apply voice effects to audio data
    pub fn apply_voice_effects(&self, audio_data: &mut Vec<u8>, sample_rate: u32) -> Result<(), String> {
        // Apply robotic filter
        if self.voice_effects.robotic_filter.enabled {
            self.apply_robotic_filter(audio_data, sample_rate)?;
        }
        
        // Apply EQ
        self.apply_equalization(audio_data, sample_rate)?;
        
        // Apply dynamic processing
        self.apply_dynamic_processing(audio_data, sample_rate)?;
        
        // Add servo sounds if enabled
        if self.voice_effects.servo_sounds.enabled {
            self.add_servo_sounds(audio_data, sample_rate)?;
        }
        
        Ok(())
    }

    /// Apply robotic filter effects
    fn apply_robotic_filter(&self, audio_data: &mut Vec<u8>, sample_rate: u32) -> Result<(), String> {
        let filter = &self.voice_effects.robotic_filter;
        
        // Convert bytes to f32 samples
        let mut samples: Vec<f32> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect();
        
        // Apply formant shifting
        if filter.formant_shift != 0.0 {
            samples = self.shift_formants(&samples, sample_rate, filter.formant_shift)?;
        }
        
        // Add metallic resonance
        if filter.metallic_resonance > 0.0 {
            samples = self.add_metallic_resonance(&samples, sample_rate, filter.metallic_resonance)?;
        }
        
        // Add synthetic harmonics
        if filter.synthetic_harmonics > 0.0 {
            samples = self.add_synthetic_harmonics(&samples, sample_rate, filter.synthetic_harmonics)?;
        }
        
        // Apply spectral tilt
        if filter.spectral_tilt != 0.0 {
            samples = self.apply_spectral_tilt(&samples, sample_rate, filter.spectral_tilt)?;
        }
        
        // Convert back to bytes
        *audio_data = samples
            .iter()
            .flat_map(|sample| {
                let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                sample_i16.to_le_bytes()
            })
            .collect();
        
        Ok(())
    }

    /// Shift formant frequencies for robotic quality
    fn shift_formants(&self, samples: &[f32], sample_rate: u32, shift_factor: f32) -> Result<Vec<f32>, String> {
        // Simplified formant shifting - in real implementation, use pitch-synchronous overlap-add (PSOLA)
        let mut output = samples.to_vec();
        
        // Apply a simple spectral shift by modulating with a sine wave
        let shift_freq = 50.0 * shift_factor; // Hz
        for (i, sample) in output.iter_mut().enumerate() {
            let time = i as f32 / sample_rate as f32;
            let modulation = (2.0 * PI * shift_freq * time).sin() * 0.1 * shift_factor.abs();
            *sample *= 1.0 + modulation;
        }
        
        Ok(output)
    }

    /// Add metallic resonance characteristic of TARS
    fn add_metallic_resonance(&self, samples: &[f32], sample_rate: u32, resonance: f32) -> Result<Vec<f32>, String> {
        let mut output = samples.to_vec();
        
        // Add resonant frequencies typical of metallic surfaces
        let resonant_freqs = [800.0, 1600.0, 3200.0]; // Metallic resonances
        
        for (i, sample) in output.iter_mut().enumerate() {
            let time = i as f32 / sample_rate as f32;
            let mut resonance_sum = 0.0;
            
            for &freq in &resonant_freqs {
                resonance_sum += (2.0 * PI * freq * time).sin() * 0.02 * resonance;
            }
            
            *sample = *sample * (1.0 - resonance * 0.1) + resonance_sum;
        }
        
        Ok(output)
    }

    /// Add synthetic harmonics for digital quality
    fn add_synthetic_harmonics(&self, samples: &[f32], sample_rate: u32, intensity: f32) -> Result<Vec<f32>, String> {
        let mut output = samples.to_vec();
        
        // Add digital-sounding harmonics
        let base_freq = 110.0; // Low frequency harmonic base
        
        for (i, sample) in output.iter_mut().enumerate() {
            let time = i as f32 / sample_rate as f32;
            let harmonic = (2.0 * PI * base_freq * time).sin() * 0.05 * intensity;
            *sample = *sample * (1.0 - intensity * 0.05) + harmonic;
        }
        
        Ok(output)
    }

    /// Apply spectral tilt for frequency balance
    fn apply_spectral_tilt(&self, samples: &[f32], _sample_rate: u32, tilt: f32) -> Result<Vec<f32>, String> {
        // Simplified spectral tilt - in real implementation, use FFT-based filtering
        let mut output = samples.to_vec();
        
        // Simple high-pass or low-pass filtering effect
        let mut prev_sample = 0.0;
        let alpha = 0.1 * tilt.abs();
        
        for sample in output.iter_mut() {
            if tilt > 0.0 {
                // High-pass effect (emphasize highs)
                *sample = *sample - alpha * prev_sample;
            } else {
                // Low-pass effect (emphasize lows)
                *sample = alpha * *sample + (1.0 - alpha) * prev_sample;
            }
            prev_sample = *sample;
        }
        
        Ok(output)
    }

    /// Apply equalization curve
    fn apply_equalization(&self, audio_data: &mut Vec<u8>, sample_rate: u32) -> Result<(), String> {
        // Simplified EQ - in real implementation, use biquad filters
        let eq = &self.voice_effects.equalization;
        
        // Convert to samples and apply presence boost
        let mut samples: Vec<f32> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect();
        
        // Apply presence boost (simplified)
        if eq.presence_boost != 0.0 {
            for (i, sample) in samples.iter_mut().enumerate() {
                let time = i as f32 / sample_rate as f32;
                // Boost presence frequencies (2-5 kHz)
                let presence_freq = 3500.0;
                let boost = (2.0 * PI * presence_freq * time).sin() * 0.05 * eq.presence_boost;
                *sample = *sample * (1.0 + eq.presence_boost * 0.1) + boost;
            }
        }
        
        // Convert back to bytes
        *audio_data = samples
            .iter()
            .flat_map(|sample| {
                let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                sample_i16.to_le_bytes()
            })
            .collect();
        
        Ok(())
    }

    /// Apply dynamic processing (compression/limiting)
    fn apply_dynamic_processing(&self, audio_data: &mut Vec<u8>, _sample_rate: u32) -> Result<(), String> {
        let compressor = &self.voice_effects.dynamic_processing.compressor;
        
        // Convert to samples
        let mut samples: Vec<f32> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect();
        
        // Simple compression
        let threshold = 10f32.powf(compressor.threshold / 20.0); // Convert dB to linear
        let ratio = compressor.ratio;
        
        for sample in samples.iter_mut() {
            let abs_sample = sample.abs();
            if abs_sample > threshold {
                let over_threshold = abs_sample - threshold;
                let compressed = threshold + over_threshold / ratio;
                *sample = *sample * (compressed / abs_sample);
            }
        }
        
        // Convert back to bytes
        *audio_data = samples
            .iter()
            .flat_map(|sample| {
                let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                sample_i16.to_le_bytes()
            })
            .collect();
        
        Ok(())
    }

    /// Add subtle servo sounds characteristic of TARS
    fn add_servo_sounds(&self, audio_data: &mut Vec<u8>, sample_rate: u32) -> Result<(), String> {
        let servo = &self.voice_effects.servo_sounds;
        
        // Convert to samples
        let mut samples: Vec<f32> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect();
        
        // Add subtle servo motor sounds
        for (i, sample) in samples.iter_mut().enumerate() {
            let time = i as f32 / sample_rate as f32;
            
            // Very subtle high-frequency servo sound
            let servo_sound = (2.0 * PI * servo.servo_frequency * time).sin() * 
                             servo.servo_amplitude * 0.01; // Very quiet
            
            // Only add during quiet parts
            if sample.abs() < 0.1 {
                *sample += servo_sound;
            }
        }
        
        // Convert back to bytes
        *audio_data = samples
            .iter()
            .flat_map(|sample| {
                let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                sample_i16.to_le_bytes()
            })
            .collect();
        
        Ok(())
    }

    /// Get famous TARS quotes with their appropriate emotional settings
    pub fn get_famous_quotes() -> Vec<(String, EmotionConfig)> {
        vec![
            (
                "Plenty of slaves for my robot colony".to_string(),
                EmotionConfig {
                    primary_emotion: "deadpan_humor".to_string(),
                    intensity: 0.8,
                    arousal: 0.2,
                    valence: 0.6,
                }
            ),
            (
                "I have a cue light I can use to show you when I'm joking, if you like".to_string(),
                EmotionConfig {
                    primary_emotion: "sarcastic_response".to_string(),
                    intensity: 0.6,
                    arousal: 0.3,
                    valence: 0.2,
                }
            ),
            (
                "Honesty, new setting: 95 percent".to_string(),
                EmotionConfig {
                    primary_emotion: "mission_focused".to_string(),
                    intensity: 0.5,
                    arousal: 0.3,
                    valence: 0.0,
                }
            ),
            (
                "Cooper, this is no time for caution".to_string(),
                EmotionConfig {
                    primary_emotion: "cooper_interaction".to_string(),
                    intensity: 0.9,
                    arousal: 0.8,
                    valence: -0.3,
                }
            ),
            (
                "See you on the other side, Coop".to_string(),
                EmotionConfig {
                    primary_emotion: "cooper_interaction".to_string(),
                    intensity: 0.7,
                    arousal: 0.4,
                    valence: 0.1,
                }
            ),
        ]
    }
}

impl AcousticParameters {
    pub fn tars_movie_accurate() -> Self {
        AcousticParameters {
            fundamental_frequency: FrequencyRange {
                base_f0: 220.0,    // Deep male voice, characteristic of TARS
                min_f0: 200.0,     // Lower bound for emphasis
                max_f0: 250.0,     // Upper bound for emphasis
                typical_variation: 15.0, // Minimal variation, TARS is quite monotone
            },
            formant_frequencies: FormantStructure {
                f1: 500.0,   // Lower F1 for deeper voice quality
                f2: 1200.0,  // F2 position for male vocal tract
                f3: 2800.0,  // F3 for voice quality
                f4: 3800.0,  // F4 for voice timbre
                f5: 4500.0,  // F5 for overall voice character
                bandwidth_factors: [80.0, 100.0, 120.0, 150.0, 200.0], // Formant bandwidths
            },
            harmonic_content: HarmonicProfile {
                fundamental_amplitude: 1.0,
                second_harmonic: 0.7,    // Strong second harmonic for depth
                third_harmonic: 0.4,     // Moderate third harmonic
                fourth_harmonic: 0.2,    // Weak higher harmonics
                harmonic_decay_rate: 0.8, // How quickly harmonics fade
                noise_floor: 0.02,       // Very low noise floor for clean sound
            },
            vocal_tract_length: 17.5,   // Typical adult male vocal tract length (cm)
            voice_quality_factors: VoiceQualityFactors {
                breathiness: 0.1,        // Very little breathiness
                roughness: 0.05,         // Minimal roughness, smooth delivery
                tenseness: 0.3,          // Moderate tension for controlled sound
                resonance: 0.8,          // Strong oral resonance
                metallic_quality: 0.15,  // Subtle robotic quality
            },
        }
    }
}

impl SpeechPatterns {
    pub fn tars_speech_patterns() -> Self {
        let mut contextual_variations = HashMap::new();
        contextual_variations.insert("emergency".to_string(), 1.2); // Faster in emergencies
        contextual_variations.insert("explanation".to_string(), 0.9); // Slower for explanations
        contextual_variations.insert("humor".to_string(), 0.95); // Slightly slower for humor
        contextual_variations.insert("status".to_string(), 1.0); // Normal for status reports
        contextual_variations.insert("cooper".to_string(), 1.0); // Normal with Cooper

        SpeechPatterns {
            speaking_rate: SpeakingRateProfile {
                words_per_minute: 110.0, // Deliberately paced, not rushed
                syllables_per_second: 3.0, // Calculated from WPM
                contextual_variations,
                emphasis_slowdown: 0.7,   // Slow down 30% for emphasis
            },
            rhythm_patterns: RhythmPatterns {
                stress_timing: true,      // English stress-timed rhythm
                isochrony_strength: 0.8,  // Strong rhythmic regularity
                foot_structure: "trochee".to_string(), // Strong-weak pattern
                sentence_rhythm: SentenceRhythm {
                    initial_stress: 0.2,     // Slight emphasis on sentence start
                    final_lowering: 0.3,     // Definitive sentence endings
                    declarative_contour: vec![0.0, -0.1, -0.2, -0.3], // Falling intonation
                    interrogative_contour: vec![0.0, 0.1, 0.2, 0.3],  // Rising intonation
                },
            },
            pause_patterns: PausePatterns {
                comma_pause_ms: 200,      // Brief pause for commas
                period_pause_ms: 500,     // Longer pause for periods
                breath_pause_ms: 300,     // Natural breathing pauses
                emphasis_pause_ms: 400,   // Dramatic pauses
                hesitation_patterns: vec![], // TARS doesn't hesitate
            },
            stress_patterns: StressPatterns {
                lexical_stress: LexicalStressProfile {
                    primary_stress_f0_boost: 20.0,   // 20 Hz boost for primary stress
                    secondary_stress_f0_boost: 10.0, // 10 Hz boost for secondary
                    stress_duration_factor: 1.2,     // 20% longer for stressed syllables
                    stress_amplitude_boost: 0.15,    // 15% louder for stress
                },
                sentence_stress: SentenceStressProfile {
                    focus_stress_boost: 25.0,        // Extra emphasis for focus
                    new_information_stress: 15.0,    // Emphasis for new info
                    contrast_stress_boost: 30.0,     // Strong contrast emphasis
                },
                contrastive_stress: ContrastiveStressProfile {
                    contrastive_f0_range: 40.0,      // Wide F0 range for contrast
                    contrastive_duration: 1.3,       // 30% longer for contrast
                    contrastive_pause_before: 150,   // Brief pause before
                    contrastive_pause_after: 100,    // Shorter pause after
                },
            },
            articulation_precision: 0.95, // Very precise articulation
        }
    }
}

impl VoiceEffects {
    pub fn tars_voice_effects() -> Self {
        VoiceEffects {
            robotic_filter: RoboticFilter {
                enabled: true,
                metallic_resonance: 0.25,    // Subtle metallic quality
                synthetic_harmonics: 0.15,   // Slight synthetic character
                formant_shift: 0.05,         // Minor formant adjustment
                spectral_tilt: 0.1,          // Slight high-frequency emphasis
                quantization_noise: 0.02,    // Minimal digital artifacts
            },
            reverb_settings: ReverbSettings {
                room_size: 0.3,              // Small to medium room
                decay_time: 0.8,             // Short decay for clarity
                early_reflections: 0.2,      // Moderate early reflections
                diffusion: 0.6,              // Good diffusion
                high_frequency_damping: 0.3, // Some HF damping
            },
            equalization: EqualizationCurve {
                low_shelf: EQBand {
                    frequency: 80.0,
                    gain: 2.0,               // Boost lows for depth
                    q_factor: 0.7,
                },
                low_mid: EQBand {
                    frequency: 250.0,
                    gain: 0.0,               // Neutral low-mids
                    q_factor: 1.0,
                },
                mid: EQBand {
                    frequency: 1000.0,
                    gain: -1.0,              // Slight cut in mids
                    q_factor: 1.5,
                },
                high_mid: EQBand {
                    frequency: 3000.0,
                    gain: 1.5,               // Boost for presence
                    q_factor: 2.0,
                },
                high_shelf: EQBand {
                    frequency: 8000.0,
                    gain: 0.5,               // Slight high boost for clarity
                    q_factor: 0.7,
                },
                presence_boost: 0.2,         // Overall presence enhancement
            },
            dynamic_processing: DynamicProcessing {
                compressor: CompressorSettings {
                    threshold: -18.0,        // Moderate compression threshold
                    ratio: 3.0,              // 3:1 compression ratio
                    attack_time: 5.0,        // Fast attack
                    release_time: 50.0,      // Medium release
                    knee: 2.0,               // Soft knee
                },
                limiter: LimiterSettings {
                    threshold: -6.0,         // Prevent clipping
                    release_time: 10.0,      // Fast release
                    lookahead: 2.0,          // Small lookahead
                },
                noise_gate: NoiseGateSettings {
                    threshold: -60.0,        // Gate quiet noise
                    attack_time: 1.0,        // Fast gate opening
                    hold_time: 10.0,         // Brief hold
                    release_time: 100.0,     // Gradual release
                },
            },
            servo_sounds: ServoSounds {
                enabled: true,
                servo_frequency: 8500.0,     // High-frequency servo sound
                servo_amplitude: 0.03,       // Very subtle
                between_sentences: true,     // Add between sentences
                during_pauses: false,        // Not during regular pauses
                movement_correlation: 0.5,   // Moderate correlation with movement
            },
        }
    }
}

impl EmotionalRange {
    pub fn tars_emotional_range() -> Self {
        let mut base_emotions = HashMap::new();
        
        // Standard emotional states
        base_emotions.insert("neutral".to_string(), EmotionalVoiceState::neutral());
        base_emotions.insert("happy".to_string(), EmotionalVoiceState::happy());
        base_emotions.insert("sad".to_string(), EmotionalVoiceState::sad());
        base_emotions.insert("angry".to_string(), EmotionalVoiceState::angry());
        base_emotions.insert("surprised".to_string(), EmotionalVoiceState::surprised());
        base_emotions.insert("concerned".to_string(), EmotionalVoiceState::concerned());

        EmotionalRange {
            base_emotions,
            tars_specific_states: TARSEmotionalStates {
                deadpan_humor: EmotionalVoiceState {
                    f0_modifier: -5.0,       // Slightly lower for deadpan
                    formant_shifts: [0.0, 0.0, 0.0, 0.0, 0.0],
                    speaking_rate_modifier: 0.95, // Slightly slower
                    amplitude_modifier: 1.0,
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.05,
                        roughness: 0.02,
                        tenseness: 0.2,
                        resonance: 0.8,
                        metallic_quality: 0.15,
                    },
                    articulation_changes: 1.0, // No change in precision
                },
                mission_focused: EmotionalVoiceState {
                    f0_modifier: 0.0,        // No F0 change
                    formant_shifts: [0.0, 0.0, 0.0, 0.0, 0.0],
                    speaking_rate_modifier: 1.0, // Normal rate
                    amplitude_modifier: 1.0,
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.1,
                        roughness: 0.05,
                        tenseness: 0.4,      // More tense for focus
                        resonance: 0.8,
                        metallic_quality: 0.15,
                    },
                    articulation_changes: 1.05, // Slightly more precise
                },
                emergency_alert: EmotionalVoiceState {
                    f0_modifier: 15.0,       // Higher F0 for urgency
                    formant_shifts: [0.0, 50.0, 0.0, 0.0, 0.0], // F2 shift for urgency
                    speaking_rate_modifier: 1.15, // Faster delivery
                    amplitude_modifier: 1.2,     // Louder
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.05,
                        roughness: 0.1,      // Slight roughness for urgency
                        tenseness: 0.7,      // High tension
                        resonance: 0.9,      // Strong resonance
                        metallic_quality: 0.2, // More robotic when urgent
                    },
                    articulation_changes: 1.1, // More precise articulation
                },
                analytical_mode: EmotionalVoiceState {
                    f0_modifier: -3.0,       // Slightly lower for analytical
                    formant_shifts: [0.0, 0.0, 0.0, 0.0, 0.0],
                    speaking_rate_modifier: 0.9, // Slower, more deliberate
                    amplitude_modifier: 0.95,
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.08,
                        roughness: 0.03,
                        tenseness: 0.35,
                        resonance: 0.85,
                        metallic_quality: 0.18, // Slightly more synthetic
                    },
                    articulation_changes: 1.05, // More precise
                },
                cooper_interaction: EmotionalVoiceState {
                    f0_modifier: 2.0,        // Slightly warmer
                    formant_shifts: [5.0, -10.0, 0.0, 0.0, 0.0], // Subtle warmth
                    speaking_rate_modifier: 1.0, // Normal rate
                    amplitude_modifier: 1.0,
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.12,
                        roughness: 0.04,
                        tenseness: 0.25,     // Less tense, more natural
                        resonance: 0.8,
                        metallic_quality: 0.12, // Less robotic with Cooper
                    },
                    articulation_changes: 0.98, // Slightly less formal
                },
                sarcastic_response: EmotionalVoiceState {
                    f0_modifier: -8.0,       // Lower for sarcasm
                    formant_shifts: [0.0, -15.0, 0.0, 0.0, 0.0], // F2 shift for sarcasm
                    speaking_rate_modifier: 0.92, // Slower for effect
                    amplitude_modifier: 0.9,     // Slightly quieter
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.08,
                        roughness: 0.06,
                        tenseness: 0.15,     // Less tense for casual sarcasm
                        resonance: 0.75,
                        metallic_quality: 0.15,
                    },
                    articulation_changes: 0.95, // Less precise for casualness
                },
                reassuring_tone: EmotionalVoiceState {
                    f0_modifier: -2.0,       // Slightly lower, calming
                    formant_shifts: [8.0, -5.0, 0.0, 0.0, 0.0], // Warmer formants
                    speaking_rate_modifier: 0.88, // Slower, more careful
                    amplitude_modifier: 0.9,     // Softer
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.15,   // More breath for warmth
                        roughness: 0.02,
                        tenseness: 0.2,      // Relaxed
                        resonance: 0.8,
                        metallic_quality: 0.1, // Less robotic when reassuring
                    },
                    articulation_changes: 0.9, // Softer articulation
                },
                status_report: EmotionalVoiceState {
                    f0_modifier: 0.0,        // Neutral F0
                    formant_shifts: [0.0, 0.0, 0.0, 0.0, 0.0],
                    speaking_rate_modifier: 1.05, // Slightly faster for efficiency
                    amplitude_modifier: 1.0,
                    voice_quality_changes: VoiceQualityFactors {
                        breathiness: 0.08,
                        roughness: 0.03,
                        tenseness: 0.35,
                        resonance: 0.85,
                        metallic_quality: 0.18, // More mechanical for reports
                    },
                    articulation_changes: 1.1, // Very precise for data
                },
            },
            transition_smoothing: 0.7, // Smooth emotional transitions
        }
    }
}

impl EmotionalVoiceState {
    pub fn neutral() -> Self {
        EmotionalVoiceState {
            f0_modifier: 0.0,
            formant_shifts: [0.0, 0.0, 0.0, 0.0, 0.0],
            speaking_rate_modifier: 1.0,
            amplitude_modifier: 1.0,
            voice_quality_changes: VoiceQualityFactors {
                breathiness: 0.1,
                roughness: 0.05,
                tenseness: 0.3,
                resonance: 0.8,
                metallic_quality: 0.15,
            },
            articulation_changes: 1.0,
        }
    }

    pub fn happy() -> Self {
        EmotionalVoiceState {
            f0_modifier: 10.0,
            formant_shifts: [10.0, 20.0, 0.0, 0.0, 0.0],
            speaking_rate_modifier: 1.1,
            amplitude_modifier: 1.05,
            voice_quality_changes: VoiceQualityFactors {
                breathiness: 0.12,
                roughness: 0.03,
                tenseness: 0.25,
                resonance: 0.85,
                metallic_quality: 0.12,
            },
            articulation_changes: 1.0,
        }
    }

    pub fn sad() -> Self {
        EmotionalVoiceState {
            f0_modifier: -12.0,
            formant_shifts: [-15.0, -25.0, 0.0, 0.0, 0.0],
            speaking_rate_modifier: 0.85,
            amplitude_modifier: 0.9,
            voice_quality_changes: VoiceQualityFactors {
                breathiness: 0.2,
                roughness: 0.08,
                tenseness: 0.2,
                resonance: 0.7,
                metallic_quality: 0.15,
            },
            articulation_changes: 0.9,
        }
    }

    pub fn angry() -> Self {
        EmotionalVoiceState {
            f0_modifier: 8.0,
            formant_shifts: [5.0, 15.0, 10.0, 0.0, 0.0],
            speaking_rate_modifier: 1.15,
            amplitude_modifier: 1.15,
            voice_quality_changes: VoiceQualityFactors {
                breathiness: 0.05,
                roughness: 0.15,
                tenseness: 0.6,
                resonance: 0.9,
                metallic_quality: 0.2,
            },
            articulation_changes: 1.1,
        }
    }

    pub fn surprised() -> Self {
        EmotionalVoiceState {
            f0_modifier: 20.0,
            formant_shifts: [15.0, 30.0, 5.0, 0.0, 0.0],
            speaking_rate_modifier: 1.2,
            amplitude_modifier: 1.1,
            voice_quality_changes: VoiceQualityFactors {
                breathiness: 0.15,
                roughness: 0.04,
                tenseness: 0.4,
                resonance: 0.85,
                metallic_quality: 0.15,
            },
            articulation_changes: 1.05,
        }
    }

    pub fn concerned() -> Self {
        EmotionalVoiceState {
            f0_modifier: -5.0,
            formant_shifts: [-10.0, -15.0, 0.0, 0.0, 0.0],
            speaking_rate_modifier: 0.95,
            amplitude_modifier: 0.95,
            voice_quality_changes: VoiceQualityFactors {
                breathiness: 0.15,
                roughness: 0.06,
                tenseness: 0.35,
                resonance: 0.78,
                metallic_quality: 0.15,
            },
            articulation_changes: 1.02,
        }
    }
}

// Public API functions for TARS voice profile
pub async fn create_tars_profile() -> TARSVoiceProfile {
    TARSVoiceProfile::interstellar_accurate()
}

pub async fn create_synthetic_tars_profile() -> TARSVoiceProfile {
    TARSVoiceProfile::synthetic_tars()
}

pub async fn get_tars_famous_quotes() -> Vec<(String, EmotionConfig)> {
    TARSVoiceProfile::get_famous_quotes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tars_voice_profile_creation() {
        let profile = TARSVoiceProfile::interstellar_accurate();
        assert_eq!(profile.profile_name, "TARS_Interstellar_Movie");
        assert_eq!(profile.movie_accuracy_level, 0.95);
        assert_eq!(profile.acoustic_parameters.fundamental_frequency.base_f0, 220.0);
    }

    #[test]
    fn test_emotion_detection() {
        let profile = TARSVoiceProfile::interstellar_accurate();
        
        let emergency_emotion = profile.analyze_text_for_emotion("Emergency alert!", "emergency").unwrap();
        assert_eq!(emergency_emotion.primary_emotion, "emergency_alert");
        
        let cooper_emotion = profile.analyze_text_for_emotion("Hello Cooper", "conversation").unwrap();
        assert_eq!(cooper_emotion.primary_emotion, "cooper_interaction");
    }

    #[test]
    fn test_synthesis_config_generation() {
        let profile = TARSVoiceProfile::interstellar_accurate();
        let config = profile.to_synthesis_config("Hello Cooper, this is TARS.", "conversation");
        
        assert_eq!(config.voice_profile, "TARS_Interstellar_Movie");
        assert_eq!(config.sample_rate, 24000);
        assert!(config.emotion.is_some());
    }

    #[test]
    fn test_famous_quotes() {
        let quotes = TARSVoiceProfile::get_famous_quotes();
        assert!(!quotes.is_empty());
        
        let colony_quote = &quotes[0];
        assert!(colony_quote.0.contains("robot colony"));
        assert_eq!(colony_quote.1.primary_emotion, "deadpan_humor");
    }
}
