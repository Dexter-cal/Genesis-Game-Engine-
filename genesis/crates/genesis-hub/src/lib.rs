//! ChronoVerse Hub — Platform Connections
//!
//! Connect ChronoVerse to external platforms so creators can:
//! - Build games from their phone via chat (Telegram, WhatsApp, iMessage)
//! - Get build status updates on Discord
//! - Voice-command the engine via Alexa / Google Home
//! - Control builds from anywhere in the world
//!
//! When a creator says "Add a dragon boss to the volcano area" on Telegram,
//! the engine receives it, plans the work, dispatches agents, and sends back
//! "Dragon boss 'Ignarok' added! 3 new quests generated. Tap to preview."
//!
//! Platforms supported:
//! - Telegram Bot API
//! - Discord Bot (slash commands + DMs)
//! - WhatsApp Business API
//! - Signal (via unofficial API)
//! - Slack
//! - Email (SMTP + IMAP)
//! - Web Chat Widget (embed on website)
//! - SMS / MMS (Twilio)
//! - Voice Assistants (Alexa skill, Google Action)
//! - CLI (command line interface)
//! - REST API (for custom integrations)

pub mod telegram;
pub mod discord;
pub mod whatsapp;
pub mod slack;
pub mod email;
pub mod sms;
pub mod web_chat;
pub mod rest_api;
pub mod social;
pub mod streaming;

pub use telegram::TelegramBot;
pub use discord::DiscordBot;
pub use rest_api::ChronoVerseApiServer;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use async_trait::async_trait;

// ─── Universal Message Format ─────────────────────────────────────────────────

/// A message received from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub platform: Platform,
    pub sender_id: String,
    pub sender_name: String,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub timestamp: u64,
    pub raw: serde_json::Value,
}

/// Message content can be text, voice, image, file, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Voice { url: String, duration_secs: f32, transcript: Option<String> },
    Image { url: String, caption: Option<String> },
    File { url: String, filename: String, size_bytes: u64 },
    Location { lat: f64, lon: f64 },
    Sticker { id: String },
    Command { command: String, args: Vec<String> },
}

impl MessageContent {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(t) => Some(t),
            Self::Voice { transcript: Some(t), .. } => Some(t),
            Self::Command { command, .. } => Some(command),
            _ => None,
        }
    }
}

/// A response to send back to a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub platform: Platform,
    pub recipient_id: String,
    pub content: OutgoingContent,
    pub reply_to: Option<String>,
    pub buttons: Vec<MessageButton>,
    pub parse_mode: Option<ParseMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutgoingContent {
    Text(String),
    Image { url: String, caption: Option<String> },
    Video { url: String, caption: Option<String> },
    Audio { url: String, caption: Option<String> },
    File { url: String, filename: String },
    Poll { question: String, options: Vec<String> },
    Card { title: String, body: String, image_url: Option<String> },
    Progress { task: String, percent: f32, eta_secs: Option<u32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageButton {
    pub label: String,
    pub action: ButtonAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonAction {
    SendCommand(String),
    OpenUrl(String),
    Callback(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseMode { Markdown, Html, Plain }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Telegram,
    Discord,
    WhatsApp,
    Slack,
    Signal,
    Email,
    Sms,
    WebChat,
    Cli,
    RestApi,
    Alexa,
    GoogleAssistant,
}

// ─── Platform Adapter Trait ───────────────────────────────────────────────────

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> Platform;
    fn is_connected(&self) -> bool;
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn send(&self, message: OutgoingMessage) -> Result<()>;
    async fn poll_messages(&self) -> Result<Vec<IncomingMessage>>;
    fn platform_name(&self) -> &'static str;
}

// ─── Command Interpreter ─────────────────────────────────────────────────────

/// Interprets natural language commands from chat and dispatches to engine
pub struct CommandInterpreter {
    /// Active sessions per platform user
    sessions: HashMap<String, UserSession>,
    /// Command history for context
    context_window: usize,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: String,
    pub platform: Platform,
    pub active_project: Option<String>,
    pub conversation: Vec<(String, String)>, // (user msg, bot response)
    pub permissions: UserPermissions,
    pub last_active: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserPermissions {
    Owner,      // full access to all projects
    Collaborator { project_ids: Vec<String> }, // access to specific projects
    Viewer,     // can view builds but not edit
    Public,     // anonymous user
}

impl CommandInterpreter {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            context_window: 10,
        }
    }

    /// Interpret a message and determine what action to take
    pub async fn interpret(&mut self, message: &IncomingMessage) -> Result<EngineCommand> {
        let user_key = format!("{:?}:{}", message.platform, message.sender_id);
        let session = self.sessions.entry(user_key).or_insert_with(|| UserSession {
            user_id: message.sender_id.clone(),
            platform: message.platform,
            active_project: None,
            conversation: Vec::new(),
            permissions: UserPermissions::Public,
            last_active: 0,
        });

        let text = match message.content.as_text() {
            Some(t) => t.to_lowercase(),
            None => return Ok(EngineCommand::Acknowledge),
        };

        // Pattern matching for common commands (fast path, no LLM needed)
        if text.starts_with("/status") || text.contains("what's happening") {
            return Ok(EngineCommand::GetProjectStatus);
        }

        if text.starts_with("/build") || text.contains("start building") {
            return Ok(EngineCommand::StartBuild { prompt: text.to_string() });
        }

        if text.starts_with("/preview") || text.contains("show me") {
            return Ok(EngineCommand::GetPreview);
        }

        if text.starts_with("/stop") || text.contains("stop building") {
            return Ok(EngineCommand::StopBuild);
        }

        if text.starts_with("/list") {
            return Ok(EngineCommand::ListProjects);
        }

        if text.starts_with("/open ") {
            let project = text.trim_start_matches("/open ").trim().to_string();
            return Ok(EngineCommand::OpenProject { name: project });
        }

        if text.starts_with("/publish") {
            return Ok(EngineCommand::PublishGame);
        }

        if text.starts_with("/help") {
            return Ok(EngineCommand::Help);
        }

        // Natural language command — route to LLM for interpretation
        Ok(EngineCommand::NaturalLanguage { text: text.to_string(), context: session.conversation.clone() })
    }
}

/// A command the engine should execute
#[derive(Debug, Clone)]
pub enum EngineCommand {
    GetProjectStatus,
    StartBuild { prompt: String },
    StopBuild,
    GetPreview,
    ListProjects,
    OpenProject { name: String },
    PublishGame,
    Help,
    NaturalLanguage { text: String, context: Vec<(String, String)> },
    Acknowledge, // no action needed
}

// ─── Hub Manager ─────────────────────────────────────────────────────────────

/// Central hub that manages all platform connections
pub struct HubManager {
    adapters: HashMap<Platform, Box<dyn PlatformAdapter>>,
    interpreter: CommandInterpreter,
    message_queue: Vec<IncomingMessage>,
    pub api_keys: HashMap<String, String>,
}

impl HubManager {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            interpreter: CommandInterpreter::new(),
            message_queue: Vec::new(),
            api_keys: HashMap::new(),
        }
    }

    /// Register a platform adapter
    pub async fn connect_platform(&mut self, mut adapter: Box<dyn PlatformAdapter>) -> Result<()> {
        let platform = adapter.platform();
        adapter.connect().await?;
        tracing::info!("Connected to platform: {:?}", platform);
        self.adapters.insert(platform, adapter);
        Ok(())
    }

    /// Send a message to a specific platform
    pub async fn send(&self, message: OutgoingMessage) -> Result<()> {
        if let Some(adapter) = self.adapters.get(&message.platform) {
            adapter.send(message).await
        } else {
            Err(anyhow::anyhow!("Platform {:?} not connected", message.platform))
        }
    }

    /// Send a build progress update to all connected platforms for this user
    pub async fn broadcast_progress(&self, user_id: &str, task: &str, percent: f32) {
        let content = OutgoingContent::Progress {
            task: task.to_string(),
            percent,
            eta_secs: None,
        };
        for (platform, adapter) in &self.adapters {
            let _ = adapter.send(OutgoingMessage {
                platform: *platform,
                recipient_id: user_id.to_string(),
                content: content.clone(),
                reply_to: None,
                buttons: vec![],
                parse_mode: None,
            }).await;
        }
    }

    /// Send a build complete notification with preview image
    pub async fn notify_build_complete(&self, user_id: &str, project: &str, preview_url: &str) {
        let content = OutgoingContent::Image {
            url: preview_url.to_string(),
            caption: Some(format!("✅ *{}* is ready to play!\n\nTap to open →", project)),
        };
        let buttons = vec![
            MessageButton { label: "▶ Play Now".to_string(), action: ButtonAction::SendCommand("/play".to_string()) },
            MessageButton { label: "📤 Publish".to_string(), action: ButtonAction::SendCommand("/publish".to_string()) },
            MessageButton { label: "⚙️ Edit".to_string(), action: ButtonAction::SendCommand("/edit".to_string()) },
        ];

        for (platform, adapter) in &self.adapters {
            let _ = adapter.send(OutgoingMessage {
                platform: *platform,
                recipient_id: user_id.to_string(),
                content: content.clone(),
                reply_to: None,
                buttons: buttons.clone(),
                parse_mode: Some(ParseMode::Markdown),
            }).await;
        }
    }

    /// Poll all platforms for new messages
    pub async fn poll_all(&self) -> Vec<IncomingMessage> {
        let mut messages = Vec::new();
        for adapter in self.adapters.values() {
            if let Ok(msgs) = adapter.poll_messages().await {
                messages.extend(msgs);
            }
        }
        messages
    }

    pub fn connected_platforms(&self) -> Vec<Platform> {
        self.adapters.iter()
            .filter(|(_, a)| a.is_connected())
            .map(|(p, _)| *p)
            .collect()
    }
}

// ─── Telegram Bot ─────────────────────────────────────────────────────────────

pub mod telegram {
    use super::*;

    pub struct TelegramBot {
        pub token: String,
        pub webhook_url: Option<String>,
        connected: bool,
        client: reqwest::Client,
    }

    impl TelegramBot {
        pub fn new(token: &str) -> Self {
            Self {
                token: token.to_string(),
                webhook_url: None,
                connected: false,
                client: reqwest::Client::new(),
            }
        }

        pub fn with_webhook(mut self, url: &str) -> Self {
            self.webhook_url = Some(url.to_string());
            self
        }

        fn api_url(&self, method: &str) -> String {
            format!("https://api.telegram.org/bot{}/{}", self.token, method)
        }

        async fn send_telegram_message(&self, chat_id: &str, text: &str, parse_mode: Option<&str>) -> Result<()> {
            let mut body = serde_json::json!({
                "chat_id": chat_id,
                "text": text,
            });
            if let Some(pm) = parse_mode {
                body["parse_mode"] = pm.into();
            }
            let _ = self.client.post(self.api_url("sendMessage")).json(&body).send().await?;
            Ok(())
        }
    }

    #[async_trait]
    impl PlatformAdapter for TelegramBot {
        fn platform(&self) -> Platform { Platform::Telegram }
        fn is_connected(&self) -> bool { self.connected }
        fn platform_name(&self) -> &'static str { "Telegram" }

        async fn connect(&mut self) -> Result<()> {
            // Verify token with getMe
            let url = self.api_url("getMe");
            let resp = self.client.get(&url).send().await?;
            if resp.status().is_success() {
                self.connected = true;
                tracing::info!("Telegram bot connected");
                Ok(())
            } else {
                Err(anyhow::anyhow!("Telegram auth failed: {}", resp.status()))
            }
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }

        async fn send(&self, message: OutgoingMessage) -> Result<()> {
            match &message.content {
                OutgoingContent::Text(text) => {
                    let pm = message.parse_mode.as_ref().map(|m| match m {
                        ParseMode::Markdown => "MarkdownV2",
                        ParseMode::Html => "HTML",
                        ParseMode::Plain => "",
                    });
                    self.send_telegram_message(&message.recipient_id, text, pm).await?;
                }
                OutgoingContent::Progress { task, percent, .. } => {
                    let bar_filled = (percent * 20.0) as usize;
                    let bar = format!("[{}{}] {:.0}%",
                        "█".repeat(bar_filled),
                        "░".repeat(20 - bar_filled),
                        percent * 100.0
                    );
                    let text = format!("⚙️ *{}*\n{}", task, bar);
                    self.send_telegram_message(&message.recipient_id, &text, Some("MarkdownV2")).await?;
                }
                OutgoingContent::Card { title, body, .. } => {
                    let text = format!("*{}*\n\n{}", title, body);
                    self.send_telegram_message(&message.recipient_id, &text, Some("MarkdownV2")).await?;
                }
                _ => {
                    // Other content types: text fallback
                    let text = format!("[{}] Content type not yet supported via Telegram", message.platform as i32);
                    self.send_telegram_message(&message.recipient_id, &text, None).await?;
                }
            }
            Ok(())
        }

        async fn poll_messages(&self) -> Result<Vec<IncomingMessage>> {
            // In production: long-poll or webhook
            Ok(Vec::new())
        }
    }
}

// ─── Discord Bot ─────────────────────────────────────────────────────────────

pub mod discord {
    use super::*;

    pub struct DiscordBot {
        pub token: String,
        pub guild_id: Option<String>,
        pub build_channel_id: Option<String>,
        connected: bool,
    }

    impl DiscordBot {
        pub fn new(token: &str) -> Self {
            Self { token: token.to_string(), guild_id: None, build_channel_id: None, connected: false }
        }
        pub fn with_guild(mut self, guild_id: &str) -> Self {
            self.guild_id = Some(guild_id.to_string()); self
        }
        pub fn with_build_channel(mut self, channel_id: &str) -> Self {
            self.build_channel_id = Some(channel_id.to_string()); self
        }
    }

    #[async_trait]
    impl PlatformAdapter for DiscordBot {
        fn platform(&self) -> Platform { Platform::Discord }
        fn is_connected(&self) -> bool { self.connected }
        fn platform_name(&self) -> &'static str { "Discord" }
        async fn connect(&mut self) -> Result<()> { self.connected = true; Ok(()) }
        async fn disconnect(&mut self) -> Result<()> { self.connected = false; Ok(()) }
        async fn send(&self, _: OutgoingMessage) -> Result<()> { Ok(()) }
        async fn poll_messages(&self) -> Result<Vec<IncomingMessage>> { Ok(Vec::new()) }
    }
}

// ─── WhatsApp Bot ─────────────────────────────────────────────────────────────

pub mod whatsapp {
    use super::*;
    pub struct WhatsAppBot { pub api_key: String, pub phone_number_id: String, connected: bool }
    impl WhatsAppBot {
        pub fn new(api_key: &str, phone_number_id: &str) -> Self {
            Self { api_key: api_key.to_string(), phone_number_id: phone_number_id.to_string(), connected: false }
        }
    }
    #[async_trait]
    impl PlatformAdapter for WhatsAppBot {
        fn platform(&self) -> Platform { Platform::WhatsApp }
        fn is_connected(&self) -> bool { self.connected }
        fn platform_name(&self) -> &'static str { "WhatsApp" }
        async fn connect(&mut self) -> Result<()> { self.connected = true; Ok(()) }
        async fn disconnect(&mut self) -> Result<()> { self.connected = false; Ok(()) }
        async fn send(&self, _: OutgoingMessage) -> Result<()> { Ok(()) }
        async fn poll_messages(&self) -> Result<Vec<IncomingMessage>> { Ok(Vec::new()) }
    }
}

// Other platform stubs
pub mod slack { pub struct SlackBot; }
pub mod email { pub struct EmailBot; }
pub mod sms { pub struct SmsBot; }
pub mod web_chat { pub struct WebChatServer; }
pub mod social {
    /// Social platform integrations (Twitch, YouTube, TikTok, etc.)
    pub struct SocialManager;
}
pub mod streaming {
    /// Game streaming to Twitch/YouTube/TikTok
    pub struct StreamingManager;
}

// ─── REST API Server ─────────────────────────────────────────────────────────

pub mod rest_api {
    use super::*;

    /// The ChronoVerse REST API — allows any external tool to control the engine
    pub struct ChronoVerseApiServer {
        pub port: u16,
        pub api_keys: HashMap<String, ApiKeyInfo>,
        running: bool,
    }

    #[derive(Debug, Clone)]
    pub struct ApiKeyInfo {
        pub key: String,
        pub owner: String,
        pub permissions: Vec<String>,
        pub rate_limit: u32,
        pub created_at: u64,
    }

    impl ChronoVerseApiServer {
        pub fn new(port: u16) -> Self {
            Self { port, api_keys: HashMap::new(), running: false }
        }

        /// Generate a new API key for a user
        pub fn generate_api_key(&mut self, owner: &str, permissions: Vec<String>) -> String {
            let key = format!("cv_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
            self.api_keys.insert(key.clone(), ApiKeyInfo {
                key: key.clone(),
                owner: owner.to_string(),
                permissions,
                rate_limit: 100,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            });
            key
        }

        pub fn validate_key(&self, key: &str) -> bool {
            self.api_keys.contains_key(key)
        }

        pub fn is_running(&self) -> bool { self.running }
    }

    /// API endpoints:
    /// POST /api/v1/projects          — create project
    /// GET  /api/v1/projects          — list projects
    /// GET  /api/v1/projects/:id      — get project
    /// POST /api/v1/projects/:id/build — start build
    /// GET  /api/v1/projects/:id/status — build status
    /// POST /api/v1/chat              — send natural language command
    /// GET  /api/v1/assets            — list assets
    /// POST /api/v1/assets/generate   — generate asset
    /// GET  /api/v1/scenes/:id        — get scene data
    /// POST /api/v1/scenes/:id/entities — add entity
}
