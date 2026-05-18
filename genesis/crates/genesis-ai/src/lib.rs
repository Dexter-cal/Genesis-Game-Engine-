//! AI System — LLM routing, provider management, TTS, STT, vision, embeddings
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ═══ PROVIDERS ═══════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum AiProvider {
    // Local
    Ollama    { url:String },
    LmStudio  { url:String },
    JanAi     { url:String },
    LlamaCpp  { url:String },
    Oobabooga { url:String },
    KoboldCpp { url:String },
    Vllm      { url:String },
    // Cloud — text
    Anthropic { model:String },
    OpenAi    { model:String },
    Google    { model:String },
    XAi       { model:String },
    Groq      { model:String },
    Cerebras  { model:String },
    DeepSeek  { model:String },
    Mistral   { model:String },
    Together  { model:String },
    Fireworks { model:String },
    Replicate { model:String },
    Cohere    { model:String },
    Perplexity{ model:String },
    OpenRouter{ model:String },
    HuggingFace{model:String},
    // Cloud GPU
    RunPod    { endpoint:String },
    VastAi    { endpoint:String },
    Lambda    { endpoint:String },
    Paperspace{ endpoint:String },
    // Specialised
    ElevenLabs,   // TTS + voice clone
    Deepgram,     // STT
    FalAi     { model:String },  // image gen (FLUX)
    Stability { model:String },  // image gen
    Custom    { name:String, url:String },
}

impl AiProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::Ollama{..}=>"Ollama", Self::LmStudio{..}=>"LM Studio",
            Self::JanAi{..}=>"Jan.ai", Self::LlamaCpp{..}=>"llama.cpp",
            Self::Oobabooga{..}=>"Oobabooga", Self::KoboldCpp{..}=>"KoboldCpp",
            Self::Vllm{..}=>"vLLM", Self::Anthropic{..}=>"Anthropic",
            Self::OpenAi{..}=>"OpenAI", Self::Google{..}=>"Google",
            Self::XAi{..}=>"xAI Grok", Self::Groq{..}=>"Groq",
            Self::Cerebras{..}=>"Cerebras", Self::DeepSeek{..}=>"DeepSeek",
            Self::Mistral{..}=>"Mistral", Self::Together{..}=>"Together AI",
            Self::Fireworks{..}=>"Fireworks", Self::Replicate{..}=>"Replicate",
            Self::Cohere{..}=>"Cohere", Self::Perplexity{..}=>"Perplexity",
            Self::OpenRouter{..}=>"OpenRouter", Self::HuggingFace{..}=>"HuggingFace",
            Self::RunPod{..}=>"RunPod", Self::VastAi{..}=>"Vast.ai",
            Self::Lambda{..}=>"Lambda Labs", Self::Paperspace{..}=>"Paperspace",
            Self::ElevenLabs=>"ElevenLabs", Self::Deepgram=>"Deepgram",
            Self::FalAi{..}=>"fal.ai", Self::Stability{..}=>"Stability AI",
            Self::Custom{name,..}=>name,
        }
    }
    pub fn is_local(&self) -> bool {
        matches!(self,Self::Ollama{..}|Self::LmStudio{..}|Self::JanAi{..}|Self::LlamaCpp{..}|Self::Oobabooga{..}|Self::KoboldCpp{..}|Self::Vllm{..})
    }
    pub fn base_url(&self) -> Option<&str> {
        match self {
            Self::Ollama{url}|Self::LmStudio{url}|Self::JanAi{url}|
            Self::LlamaCpp{url}|Self::Oobabooga{url}|Self::KoboldCpp{url}|
            Self::Vllm{url}|Self::Custom{url,..}=>Some(url),
            Self::Anthropic{..}=>Some("https://api.anthropic.com"),
            Self::OpenAi{..}=>Some("https://api.openai.com"),
            Self::Google{..}=>Some("https://generativelanguage.googleapis.com"),
            Self::Groq{..}=>Some("https://api.groq.com/openai"),
            Self::Cerebras{..}=>Some("https://api.cerebras.ai/v1"),
            Self::DeepSeek{..}=>Some("https://api.deepseek.com"),
            Self::Mistral{..}=>Some("https://api.mistral.ai/v1"),
            Self::Together{..}=>Some("https://api.together.xyz/v1"),
            Self::Fireworks{..}=>Some("https://api.fireworks.ai/inference/v1"),
            Self::ElevenLabs=>Some("https://api.elevenlabs.io"),
            Self::Deepgram=>Some("https://api.deepgram.com"),
            _=>None,
        }
    }
    pub fn supports_vision(&self) -> bool {
        matches!(self,Self::Anthropic{..}|Self::OpenAi{..}|Self::Google{..}|Self::Ollama{..})
    }
    pub fn supports_streaming(&self) -> bool {
        !matches!(self,Self::Deepgram|Self::FalAi{..}|Self::Stability{..})
    }
}

// ═══ API KEY MANAGER (AES-256 encrypted) ════════════════════════
pub struct ApiKeyStore {
    keys:         HashMap<String,EncryptedKey>,
    pub providers_configured: Vec<String>,
}

#[derive(Debug,Clone)]
struct EncryptedKey {
    ciphertext: Vec<u8>,
    nonce:      Vec<u8>,
}

impl ApiKeyStore {
    pub fn new() -> Self { Self { keys:HashMap::new(), providers_configured:Vec::new() } }

    pub fn store(&mut self, provider:&str, key:&str) {
        // Real impl: AES-256-GCM encrypt before storing
        // For now store as bytes (real impl would encrypt)
        self.keys.insert(provider.to_string(), EncryptedKey { ciphertext:key.as_bytes().to_vec(), nonce:Vec::new() });
        if !self.providers_configured.contains(&provider.to_string()) {
            self.providers_configured.push(provider.to_string());
        }
        tracing::info!("API key stored for: {}", provider);
    }

    pub fn get(&self, provider:&str) -> Option<String> {
        self.keys.get(provider).map(|k| String::from_utf8_lossy(&k.ciphertext).into_owned())
    }

    pub fn has(&self, provider:&str) -> bool { self.keys.contains_key(provider) }
    pub fn remove(&mut self, provider:&str) { self.keys.remove(provider); self.providers_configured.retain(|p|p!=provider); }
    pub fn count(&self) -> usize { self.keys.len() }
}

// ═══ REQUEST / RESPONSE ══════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChatMessage {
    pub role:    ChatRole,
    pub content: MessageContent,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ChatRole { System, User, Assistant, Tool }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MessageContent {
    Text(String),
    Multi(Vec<ContentPart>),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ContentPart {
    Text(String),
    Image { data:String, mime:String }, // base64
    Audio { data:String, mime:String },
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChatRequest {
    pub model:        String,
    pub messages:     Vec<ChatMessage>,
    pub system:       Option<String>,
    pub temperature:  f32,
    pub max_tokens:   u32,
    pub stream:       bool,
    pub tools:        Vec<serde_json::Value>,
    pub stop:         Vec<String>,
    pub json_mode:    bool,
    pub seed:         Option<u64>,
    pub top_p:        f32,
    pub top_k:        Option<u32>,
    pub frequency_penalty: f32,
    pub presence_penalty:  f32,
    pub timeout_secs: u32,
}

impl ChatRequest {
    pub fn simple(model:&str, system:&str, user:&str) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![
                ChatMessage { role:ChatRole::User, content:MessageContent::Text(user.to_string()) }
            ],
            system: Some(system.to_string()),
            temperature: 0.7, max_tokens: 2048, stream: false,
            tools: Vec::new(), stop: Vec::new(), json_mode: false, seed: None,
            top_p: 0.95, top_k: None, frequency_penalty: 0.0, presence_penalty: 0.0,
            timeout_secs: 60,
        }
    }
    pub fn with_json(mut self) -> Self { self.json_mode = true; self }
    pub fn with_temp(mut self, t:f32) -> Self { self.temperature = t; self }
    pub fn with_max_tokens(mut self, n:u32) -> Self { self.max_tokens = n; self }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChatResponse {
    pub content:       String,
    pub model:         String,
    pub provider:      String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens:  u32,
    pub cost_usd:      f64,
    pub latency_ms:    u32,
    pub finish_reason: FinishReason,
    pub tool_calls:    Vec<ToolCallResponse>,
    pub timestamp:     DateTime<Utc>,
    pub cached:        bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FinishReason { Stop, MaxTokens, ToolCall, Error, ContentFilter }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ToolCallResponse { pub id:String, pub name:String, pub args:serde_json::Value }

impl ChatResponse {
    pub fn parse_json<T:serde::de::DeserializeOwned>(&self) -> Result<T,serde_json::Error> {
        let clean = self.content.trim().trim_start_matches("```json").trim_end_matches("```").trim();
        serde_json::from_str(clean)
    }
    pub fn is_ok(&self) -> bool { !matches!(self.finish_reason, FinishReason::Error) }
}

// ═══ MODEL DOWNLOAD PIPELINE ═════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ModelDownload {
    pub id:           String,
    pub name:         String,
    pub provider:     String,
    pub size_gb:      f32,
    pub quantization: String,
    pub status:       DownloadStatus,
    pub progress:     f32,
    pub local_path:   Option<String>,
    pub vram_required:f32,
    pub ram_required: f32,
    pub context_len:  u32,
    pub tags:         Vec<String>,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum DownloadStatus { NotDownloaded, Queued, Downloading, Verifying, Ready, Failed(String), Outdated }

impl ModelDownload {
    pub fn is_usable(&self) -> bool { self.status == DownloadStatus::Ready }
}

// ═══ TTS / STT ═══════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TtsRequest {
    pub text:       String,
    pub voice_id:   String,
    pub model:      String,
    pub stability:  f32,
    pub similarity: f32,
    pub speed:      f32,
    pub pitch:      f32,
    pub format:     AudioFormat,
    pub sample_rate:u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AudioFormat { Mp3, Wav, Ogg, Flac, Pcm }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SttRequest {
    pub audio_data:  Vec<u8>,
    pub format:      AudioFormat,
    pub language:    Option<String>,
    pub model:       String,
    pub timestamps:  bool,
    pub speaker_diarization: bool,
    pub punctuation: bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SttResponse {
    pub text:       String,
    pub words:      Vec<WordTimestamp>,
    pub confidence: f32,
    pub language:   String,
    pub duration_secs: f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WordTimestamp { pub word:String, pub start:f32, pub end:f32, pub confidence:f32 }

// ═══ EMBEDDING ═══════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EmbeddingRequest { pub text:String, pub model:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EmbeddingResponse { pub embedding:Vec<f32>, pub model:String, pub tokens:u32 }

impl EmbeddingResponse {
    pub fn cosine_sim(&self, other:&[f32]) -> f32 {
        let dot:f32 = self.embedding.iter().zip(other.iter()).map(|(a,b)|a*b).sum();
        let na:f32  = self.embedding.iter().map(|x|x*x).sum::<f32>().sqrt();
        let nb:f32  = other.iter().map(|x|x*x).sum::<f32>().sqrt();
        if na==0.0||nb==0.0 { 0.0 } else { dot/(na*nb) }
    }
}

// ═══ AI RUNTIME ══════════════════════════════════════════════════
pub struct AiRuntime {
    pub primary_provider:    AiProvider,
    pub fallback_providers:  Vec<AiProvider>,
    pub key_store:           ApiKeyStore,
    pub models:              Vec<ModelDownload>,
    pub request_log:         std::collections::VecDeque<ChatResponse>,
    pub log_capacity:        usize,
    pub total_requests:      u64,
    pub total_tokens:        u64,
    pub total_cost_usd:      f64,
    pub total_errors:        u32,
    pub cache:               HashMap<u64,ChatResponse>,
    pub cache_enabled:       bool,
    pub ollama_available:    bool,
    pub hardware_tier:       HardwareTier,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum HardwareTier { Minimal, Low, Medium, High, Ultra, AppleSilicon }

impl HardwareTier {
    pub fn recommended_model(&self) -> &'static str {
        match self {
            Self::Minimal       => "tinyllama:1.1b",
            Self::Low           => "phi4-mini:3.8b",
            Self::Medium        => "qwen2.5:7b",
            Self::High          => "mixtral:8x7b",
            Self::Ultra         => "llama3.3:70b",
            Self::AppleSilicon  => "mlx-qwen2.5:7b",
        }
    }
}

impl AiRuntime {
    pub fn new(tier:HardwareTier) -> Self {
        let model = tier.recommended_model();
        Self {
            primary_provider: AiProvider::Ollama { url:"http://localhost:11434".to_string() },
            fallback_providers: vec![
                AiProvider::Anthropic { model:"claude-sonnet-4-6".to_string() },
                AiProvider::OpenAi    { model:"gpt-4o".to_string() },
                AiProvider::Groq      { model:"llama-3.1-70b-versatile".to_string() },
                AiProvider::DeepSeek  { model:"deepseek-chat".to_string() },
            ],
            key_store: ApiKeyStore::new(),
            models: Self::default_models(&tier),
            request_log: std::collections::VecDeque::new(),
            log_capacity: 500,
            total_requests: 0, total_tokens: 0,
            total_cost_usd: 0.0, total_errors: 0,
            cache: HashMap::new(), cache_enabled: true,
            ollama_available: false, hardware_tier: tier,
        }
    }

    fn default_models(tier:&HardwareTier) -> Vec<ModelDownload> {
        vec![
            ModelDownload { id:"tinyllama:1.1b".to_string(), name:"TinyLlama 1.1B Q4".to_string(), provider:"ollama".to_string(), size_gb:0.6, quantization:"Q4_K_M".to_string(), status:DownloadStatus::NotDownloaded, progress:0.0, local_path:None, vram_required:1.0, ram_required:2.0, context_len:4096, tags:vec!["small".to_string(),"fast".to_string()] },
            ModelDownload { id:"phi4-mini:3.8b".to_string(), name:"Phi-4 Mini 3.8B Q4".to_string(), provider:"ollama".to_string(), size_gb:2.3, quantization:"Q4_K_M".to_string(), status:DownloadStatus::NotDownloaded, progress:0.0, local_path:None, vram_required:3.0, ram_required:5.0, context_len:8192, tags:vec!["small".to_string(),"quality".to_string()] },
            ModelDownload { id:"qwen2.5:7b".to_string(), name:"Qwen 2.5 7B Q4".to_string(), provider:"ollama".to_string(), size_gb:4.7, quantization:"Q4_K_M".to_string(), status:DownloadStatus::NotDownloaded, progress:0.0, local_path:None, vram_required:5.0, ram_required:8.0, context_len:32768, tags:vec!["balanced".to_string(),"recommended".to_string()] },
            ModelDownload { id:"mixtral:8x7b".to_string(), name:"Mixtral 8x7B MoE Q4".to_string(), provider:"ollama".to_string(), size_gb:26.0, quantization:"Q4_K_M".to_string(), status:DownloadStatus::NotDownloaded, progress:0.0, local_path:None, vram_required:12.0, ram_required:48.0, context_len:32768, tags:vec!["large".to_string(),"capable".to_string()] },
            ModelDownload { id:"llama3.3:70b".to_string(), name:"Llama 3.3 70B Q4".to_string(), provider:"ollama".to_string(), size_gb:43.0, quantization:"Q4_K_M".to_string(), status:DownloadStatus::NotDownloaded, progress:0.0, local_path:None, vram_required:24.0, ram_required:64.0, context_len:131072, tags:vec!["flagship".to_string(),"best".to_string()] },
        ]
    }

    pub fn set_key(&mut self, provider:&str, key:&str) { self.key_store.store(provider, key); }
    pub fn has_key(&self, provider:&str) -> bool { self.key_store.has(provider) }

    pub fn record_response(&mut self, resp:ChatResponse) {
        self.total_requests += 1;
        self.total_tokens   += resp.total_tokens as u64;
        self.total_cost_usd += resp.cost_usd;
        if matches!(resp.finish_reason, FinishReason::Error) { self.total_errors += 1; }
        self.request_log.push_back(resp);
        while self.request_log.len() > self.log_capacity { self.request_log.pop_front(); }
    }

    pub fn avg_latency_ms(&self) -> f32 {
        if self.request_log.is_empty() { return 0.0; }
        self.request_log.iter().map(|r|r.latency_ms as f32).sum::<f32>() / self.request_log.len() as f32
    }

    pub fn available_providers(&self) -> Vec<&AiProvider> {
        let mut avail = vec![&self.primary_provider];
        for fb in &self.fallback_providers {
            if let Some(key_name) = provider_key_name(fb) {
                if self.key_store.has(key_name) { avail.push(fb); }
            } else { avail.push(fb); }
        }
        avail
    }

    pub fn model_for_tier(&self) -> &str { self.hardware_tier.recommended_model() }
    pub fn request_count(&self) -> u64 { self.total_requests }
    pub fn cost_today(&self) -> f64 { self.total_cost_usd }
    pub fn error_rate(&self) -> f32 {
        if self.total_requests == 0 { 0.0 }
        else { self.total_errors as f32 / self.total_requests as f32 }
    }
}

fn provider_key_name(p:&AiProvider) -> Option<&'static str> {
    match p {
        AiProvider::Anthropic{..}  => Some("ANTHROPIC_API_KEY"),
        AiProvider::OpenAi{..}     => Some("OPENAI_API_KEY"),
        AiProvider::Google{..}     => Some("GOOGLE_AI_API_KEY"),
        AiProvider::Groq{..}       => Some("GROQ_API_KEY"),
        AiProvider::Cerebras{..}   => Some("CEREBRAS_API_KEY"),
        AiProvider::DeepSeek{..}   => Some("DEEPSEEK_API_KEY"),
        AiProvider::Mistral{..}    => Some("MISTRAL_API_KEY"),
        AiProvider::Together{..}   => Some("TOGETHER_API_KEY"),
        AiProvider::Fireworks{..}  => Some("FIREWORKS_API_KEY"),
        AiProvider::ElevenLabs     => Some("ELEVENLABS_API_KEY"),
        AiProvider::Deepgram       => Some("DEEPGRAM_API_KEY"),
        AiProvider::FalAi{..}      => Some("FAL_API_KEY"),
        _ => None,
    }
}
extern crate tracing;
pub mod nodes;
pub mod mcp;
