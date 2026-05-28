//! Voice Command Processing for Genesis AI
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub transcript: String,
    pub detected_intent: CommandIntent,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandIntent {
    SpawnEntity,
    ChangeLighting,
    ModifyPhysics,
    RunPerformanceCheck,
    Unknown,
}

pub fn parse_voice_command(transcript: &str) -> VoiceCommand {
    let lower = transcript.to_lowercase();
    let intent = if lower.contains("spawn") || lower.contains("create") {
        CommandIntent::SpawnEntity
    } else if lower.contains("light") || lower.contains("sun") {
        CommandIntent::ChangeLighting
    } else if lower.contains("gravity") || lower.contains("physics") {
        CommandIntent::ModifyPhysics
    } else if lower.contains("test") || lower.contains("performance") {
        CommandIntent::RunPerformanceCheck
    } else {
        CommandIntent::Unknown
    };

    VoiceCommand {
        transcript: transcript.to_string(),
        detected_intent: intent,
        confidence: 0.92,
    }
}
