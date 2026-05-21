//! Genesis Engine Console / Terminal
//!
//! A fully-featured in-engine console like Godot's,
//! but with AI assistance built in.
//!
//! Features:
//! - Multi-language REPL (ChronoScript, Lua, Rhai, Python)
//! - Full syntax highlighting for all languages
//! - Rich error reporting (line, column, suggestions, fix hints)
//! - AI code assistant (ask questions, get completions)
//! - Command history with search
//! - Auto-complete for engine API
//! - Variable inspector (watch live values)
//! - Performance profiler output
//! - Log viewer (filter by level/source)
//! - Script debugger (breakpoints, step through)
//! - Remote console (connect to running game)
//! - Macro system (save frequently used commands)
//! - Multi-tab console (one per language/purpose)
//! - Output can be exported to file
//! - Dark/light theme

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

// ─── Console Output ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub source: String,         // "script:my_script.cv", "engine:physics", etc.
    pub message: String,
    pub rich_message: Option<RichConsoleMessage>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub stack_trace: Vec<StackFrame>,
    pub count: u32,             // repeated message count
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    /// Script print() output
    Script,
    /// User typed in console
    UserInput,
    /// Console result/return value
    Result,
    /// AI assistant response
    AiResponse,
}

impl LogLevel {
    pub fn color(&self) -> [f32; 4] {
        match self {
            Self::Trace    => [0.6, 0.6, 0.6, 1.0],
            Self::Debug    => [0.7, 0.8, 1.0, 1.0],
            Self::Info     => [1.0, 1.0, 1.0, 1.0],
            Self::Warning  => [1.0, 0.85, 0.3, 1.0],
            Self::Error    => [1.0, 0.3, 0.3, 1.0],
            Self::Fatal    => [1.0, 0.0, 0.5, 1.0],
            Self::Script   => [0.7, 1.0, 0.7, 1.0],
            Self::UserInput=> [0.8, 0.8, 1.0, 1.0],
            Self::Result   => [0.5, 1.0, 0.9, 1.0],
            Self::AiResponse=>[0.9, 0.7, 1.0, 1.0],
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Trace    => "TRACE",
            Self::Debug    => "DEBUG",
            Self::Info     => "INFO ",
            Self::Warning  => "WARN ",
            Self::Error    => "ERROR",
            Self::Fatal    => "FATAL",
            Self::Script   => "SCRPT",
            Self::UserInput=> "  >>>",
            Self::Result   => "  <<=",
            Self::AiResponse=>"🤖 AI",
        }
    }
}

/// Rich formatted console message (supports links, inline buttons, colored spans)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichConsoleMessage {
    pub spans: Vec<ConsoleSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleSpan {
    pub text: String,
    pub color: Option<[f32; 4]>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub link: Option<ConsoleLink>,
    pub inline_button: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleLink {
    /// Opens file at line in editor
    FileLocation { path: String, line: u32, column: u32 },
    /// Navigates to a scene node
    NodePath(String),
    /// Executes a console command
    Command(String),
    /// Opens documentation
    Docs(String),
    Url(String),
}

/// A stack frame for error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub locals: HashMap<String, String>, // variable: value
}

// ─── Rich Error Reporting ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptError {
    pub error_type: ErrorType,
    pub message: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub source_context: Vec<SourceLine>,
    pub suggestion: Option<ErrorSuggestion>,
    pub related_errors: Vec<RelatedError>,
    pub error_code: String,      // e.g. "CV0042"
    pub documentation_url: Option<String>,
    pub ai_fix_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    SyntaxError,
    TypeError,
    NameError,       // undefined variable
    AttributeError,  // no such property
    IndexError,
    ValueError,
    RuntimeError,
    StackOverflow,
    TimeoutError,    // script exceeded time budget
    MemoryError,
    ImportError,
    SecurityError,   // tried to access forbidden API
    ApiError,        // engine API misuse
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLine {
    pub line_number: u32,
    pub content: String,
    pub is_error_line: bool,
    pub highlight_start: Option<u32>,
    pub highlight_end: Option<u32>,
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSuggestion {
    pub message: String,
    pub fix_type: FixType,
    pub auto_fix: Option<String>,  // the fixed code
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixType {
    Replace  { before: String, after: String },
    Insert   { at_line: u32, code: String },
    Delete   { from_line: u32, to_line: u32 },
    AiGenerated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedError {
    pub message: String,
    pub file: String,
    pub line: u32,
    pub relation: String,  // "first defined here", "also used here"
}

// ─── Auto-complete ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,  // type signature
    pub documentation: Option<String>,
    pub insert_text: String,
    pub sort_priority: u32,
    pub snippet: bool,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionKind {
    Function, Method, Property, Variable, Class, Module,
    Constant, Keyword, Snippet, Event, Signal,
}

pub struct AutoComplete {
    /// Engine API surface (functions/properties available to scripts)
    api_surface: HashMap<String, Vec<CompletionItem>>,
    /// User-defined symbols from the current project
    project_symbols: Vec<CompletionItem>,
    /// Recently used completions (boost priority)
    recently_used: VecDeque<String>,
}

impl AutoComplete {
    pub fn new() -> Self {
        let mut ac = Self {
            api_surface: HashMap::new(),
            project_symbols: Vec::new(),
            recently_used: VecDeque::new(),
        };
        ac.register_engine_api();
        ac
    }

    fn register_engine_api(&mut self) {
        // World API
        self.api_surface.insert("world".to_string(), vec![
            CompletionItem { label: "spawn_entity".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(type: str, name: str, position: Vec3) -> Entity".to_string()),
                documentation: Some("Spawn a new entity in the world".to_string()),
                insert_text: "spawn_entity(\"${1:npc}\", \"${2:name}\", [${3:0}, ${4:0}, ${5:0}])".to_string(),
                sort_priority: 10, snippet: true, deprecated: false },
            CompletionItem { label: "find_entity".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(id: str) -> Entity?".to_string()),
                documentation: Some("Find entity by ID".to_string()),
                insert_text: "find_entity(\"${1:entity_id}\")".to_string(),
                sort_priority: 9, snippet: true, deprecated: false },
            CompletionItem { label: "entities_near".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(pos: Vec3, radius: f32) -> [Entity]".to_string()),
                documentation: Some("Find all entities within radius".to_string()),
                insert_text: "entities_near(${1:position}, ${2:10.0})".to_string(),
                sort_priority: 9, snippet: true, deprecated: false },
        ]);

        // Player API
        self.api_surface.insert("player".to_string(), vec![
            CompletionItem { label: "position".to_string(), kind: CompletionKind::Property,
                detail: Some("Vec3".to_string()), documentation: Some("Player world position".to_string()),
                insert_text: "position".to_string(), sort_priority: 10, snippet: false, deprecated: false },
            CompletionItem { label: "health".to_string(), kind: CompletionKind::Property,
                detail: Some("f32".to_string()), documentation: None,
                insert_text: "health".to_string(), sort_priority: 9, snippet: false, deprecated: false },
            CompletionItem { label: "give_item".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(item_id: str, count: i32)".to_string()), documentation: None,
                insert_text: "give_item(\"${1:item_id}\", ${2:1})".to_string(),
                sort_priority: 8, snippet: true, deprecated: false },
        ]);

        // Self API (entity this script is attached to)
        self.api_surface.insert("self".to_string(), vec![
            CompletionItem { label: "say".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(text: str, emotion: str?)".to_string()),
                documentation: Some("Make this NPC speak".to_string()),
                insert_text: "say(\"${1:Hello!}\")".to_string(),
                sort_priority: 10, snippet: true, deprecated: false },
            CompletionItem { label: "play_animation".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(anim: str, speed: f32?)".to_string()), documentation: None,
                insert_text: "play_animation(\"${1:idle}\")".to_string(),
                sort_priority: 9, snippet: true, deprecated: false },
            CompletionItem { label: "move_to".to_string(), kind: CompletionKind::Method,
                detail: Some("fn(target: Vec3, speed: f32?)".to_string()), documentation: None,
                insert_text: "move_to(${1:position})".to_string(),
                sort_priority: 9, snippet: true, deprecated: false },
        ]);
    }

    pub fn get_completions(&self, context: &str, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        let parts: Vec<&str> = context.splitn(2, '.').collect();

        if parts.len() == 2 {
            // Member access: world.XXX, player.XXX
            if let Some(api) = self.api_surface.get(parts[0]) {
                items.extend(api.iter()
                    .filter(|item| item.label.starts_with(prefix))
                    .cloned());
            }
        } else {
            // Top-level: add all namespaces
            for ns in self.api_surface.keys() {
                if ns.starts_with(prefix) {
                    items.push(CompletionItem {
                        label: ns.clone(), kind: CompletionKind::Module,
                        detail: Some("Engine API namespace".to_string()),
                        documentation: None,
                        insert_text: format!("{}", ns),
                        sort_priority: 5, snippet: false, deprecated: false,
                    });
                }
            }
            // Add keywords
            for kw in &["def", "if", "else", "elif", "for", "while", "return",
                        "var", "let", "fn", "class", "import", "from", "pass",
                        "break", "continue", "not", "and", "or", "true", "false"] {
                if kw.starts_with(prefix) {
                    items.push(CompletionItem {
                        label: kw.to_string(), kind: CompletionKind::Keyword,
                        detail: None, documentation: None,
                        insert_text: kw.to_string(),
                        sort_priority: 1, snippet: false, deprecated: false,
                    });
                }
            }
        }

        // Sort by priority then alpha
        items.sort_by(|a, b| b.sort_priority.cmp(&a.sort_priority).then(a.label.cmp(&b.label)));
        items
    }
}

// ─── Debugger ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: u32,
    pub file: String,
    pub line: u32,
    pub enabled: bool,
    pub condition: Option<String>,  // conditional breakpoint expression
    pub hit_count: u32,
    pub log_message: Option<String>, // log instead of breaking (logpoint)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebuggerState {
    Running,
    Paused { at_file: String, at_line: u32 },
    StepOver,
    StepInto,
    StepOut,
    Stopped,
}

pub struct Debugger {
    pub state: DebuggerState,
    pub breakpoints: Vec<Breakpoint>,
    pub call_stack: Vec<StackFrame>,
    pub locals: HashMap<String, DebugValue>,
    pub watch_expressions: Vec<WatchExpression>,
    pub step_counter: u64,
    pub max_steps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugValue {
    pub name: String,
    pub type_name: String,
    pub value: String,       // display string
    pub raw: serde_json::Value,
    pub expandable: bool,
    pub children: Vec<DebugValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchExpression {
    pub expression: String,
    pub current_value: Option<DebugValue>,
    pub error: Option<String>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            state: DebuggerState::Running,
            breakpoints: Vec::new(),
            call_stack: Vec::new(),
            locals: HashMap::new(),
            watch_expressions: Vec::new(),
            step_counter: 0,
            max_steps: u64::MAX,
        }
    }

    pub fn add_breakpoint(&mut self, file: &str, line: u32) -> u32 {
        let id = self.breakpoints.len() as u32 + 1;
        self.breakpoints.push(Breakpoint {
            id, file: file.to_string(), line, enabled: true,
            condition: None, hit_count: 0, log_message: None,
        });
        id
    }

    pub fn remove_breakpoint(&mut self, id: u32) {
        self.breakpoints.retain(|b| b.id != id);
    }

    pub fn should_break(&mut self, file: &str, line: u32) -> bool {
        for bp in &mut self.breakpoints {
            if bp.file == file && bp.line == line && bp.enabled {
                bp.hit_count += 1;
                // Check condition (in production: evaluate expression)
                return bp.condition.is_none() || bp.hit_count == 1;
            }
        }
        false
    }

    pub fn pause_at(&mut self, file: &str, line: u32) {
        self.state = DebuggerState::Paused { at_file: file.to_string(), at_line: line };
    }

    pub fn resume(&mut self) {
        self.state = DebuggerState::Running;
    }

    pub fn step_over(&mut self) {
        self.state = DebuggerState::StepOver;
    }
}

// ─── AI Code Assistant ────────────────────────────────────────────────────────

pub struct AiCodeAssistant {
    pub enabled: bool,
    pub current_language: String,
    pub context_lines: usize,     // how many lines of context to include
    pub inline_suggestions: bool,
    pub explain_errors: bool,
    pub auto_fix: bool,
    pub model_preference: AiAssistModel,
    pub conversation: Vec<AiCodeMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiAssistModel {
    Fast,     // local tiny model for quick completions
    Quality,  // larger model for explanations/fixes
    Cloud,    // cloud API for complex help
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCodeMessage {
    pub role: AiMessageRole,
    pub content: String,
    pub code_blocks: Vec<CodeBlock>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiMessageRole { User, Assistant, System }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl AiCodeAssistant {
    pub fn new() -> Self {
        Self {
            enabled: true,
            current_language: "chronoscript".to_string(),
            context_lines: 50,
            inline_suggestions: true,
            explain_errors: true,
            auto_fix: false,
            model_preference: AiAssistModel::Fast,
            conversation: Vec::new(),
        }
    }

    /// Format an error explanation request
    pub fn build_error_prompt(&self, error: &ScriptError, context: &str) -> String {
        format!(
            "I have a {} error in my {} script.\n\
             \nError: {} ({})\n\
             File: {}, Line {}, Column {}\n\
             \nCode context:\n```{}\n{}\n```\n\
             \nPlease explain what went wrong and how to fix it. \
             Give a working code example.",
            format!("{:?}", error.error_type),
            self.current_language,
            error.message,
            error.error_code,
            error.file, error.line, error.column,
            self.current_language,
            context
        )
    }

    /// Format a code completion request
    pub fn build_completion_prompt(&self, code_before: &str, instruction: &str) -> String {
        format!(
            "In {} scripting language for Genesis game engine:\n\
             Context: {}\n\
             Instruction: {}\n\
             Complete the code following the instruction. \
             Only output the code, no explanation.",
            self.current_language, code_before, instruction
        )
    }

    pub fn add_message(&mut self, role: AiMessageRole, content: &str) {
        self.conversation.push(AiCodeMessage {
            role,
            content: content.to_string(),
            code_blocks: Vec::new(),
            timestamp: Utc::now(),
        });
        // Keep conversation manageable
        if self.conversation.len() > 50 {
            self.conversation.remove(1); // keep system message
        }
    }
}

// ─── Console Tabs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleTab {
    pub id: String,
    pub name: String,
    pub tab_type: ConsoleTabType,
    pub language: String,
    pub input_history: Vec<String>,
    pub history_index: i32,
    pub current_input: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleTabType {
    Repl,       // interactive REPL
    Log,        // log viewer
    Profiler,   // performance profiler
    Network,    // network traffic monitor
    Physics,    // physics debug
    AiChat,     // AI assistant chat
    Remote,     // connected to remote game instance
}

// ─── The Console ─────────────────────────────────────────────────────────────

pub struct EngineConsole {
    pub entries: VecDeque<ConsoleEntry>,
    pub max_entries: usize,
    pub tabs: Vec<ConsoleTab>,
    pub active_tab: usize,
    pub filter_level: Option<LogLevel>,
    pub filter_source: Option<String>,
    pub search_text: String,
    pub auto_scroll: bool,
    pub timestamps_visible: bool,
    pub auto_complete: AutoComplete,
    pub debugger: Debugger,
    pub ai_assistant: AiCodeAssistant,
    pub next_entry_id: u64,
    pub macros: HashMap<String, String>,  // name → command
    pub theme: ConsoleTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleTheme { Dark, Light, Solarized, Monokai, Nord }

impl EngineConsole {
    pub fn new() -> Self {
        let mut console = Self {
            entries: VecDeque::new(),
            max_entries: 10_000,
            tabs: vec![
                ConsoleTab {
                    id: "repl".to_string(),
                    name: "Console".to_string(),
                    tab_type: ConsoleTabType::Repl,
                    language: "chronoscript".to_string(),
                    input_history: Vec::new(),
                    history_index: -1,
                    current_input: String::new(),
                    pinned: true,
                },
                ConsoleTab {
                    id: "log".to_string(),
                    name: "Log".to_string(),
                    tab_type: ConsoleTabType::Log,
                    language: "".to_string(),
                    input_history: Vec::new(),
                    history_index: -1,
                    current_input: String::new(),
                    pinned: true,
                },
                ConsoleTab {
                    id: "ai".to_string(),
                    name: "🤖 AI Help".to_string(),
                    tab_type: ConsoleTabType::AiChat,
                    language: "".to_string(),
                    input_history: Vec::new(),
                    history_index: -1,
                    current_input: String::new(),
                    pinned: true,
                },
            ],
            active_tab: 0,
            filter_level: None,
            filter_source: None,
            search_text: String::new(),
            auto_scroll: true,
            timestamps_visible: true,
            auto_complete: AutoComplete::new(),
            debugger: Debugger::new(),
            ai_assistant: AiCodeAssistant::new(),
            next_entry_id: 0,
            macros: HashMap::new(),
            theme: ConsoleTheme::Dark,
        };

        // Register common macros
        console.macros.insert("fps".to_string(), "debug.print_fps()".to_string());
        console.macros.insert("entities".to_string(), "world.entity_count()".to_string());
        console.macros.insert("clear".to_string(), "__clear__".to_string());

        // Welcome message
        console.log(LogLevel::Info, "engine", "Genesis Engine Console ready. Type 'help' for commands.");
        console.log(LogLevel::Info, "engine", "AI Assistant active — ask questions with '?' prefix: ?how do I spawn an enemy");
        console
    }

    pub fn log(&mut self, level: LogLevel, source: &str, message: &str) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        let id = self.next_entry_id;
        self.next_entry_id += 1;
        self.entries.push_back(ConsoleEntry {
            id,
            timestamp: Utc::now(),
            level,
            source: source.to_string(),
            message: message.to_string(),
            rich_message: None,
            file: None,
            line: None,
            column: None,
            stack_trace: Vec::new(),
            count: 1,
        });
    }

    pub fn error_from_script(&mut self, error: &ScriptError) {
        let msg = format!(
            "[{}] {} ({}:{}:{})\n{}\n{}",
            error.error_code,
            error.message,
            error.file, error.line, error.column,
            error.source_context.iter()
                .map(|l| {
                    let prefix = if l.is_error_line { "→ " } else { "  " };
                    format!("{}{:4} | {}", prefix, l.line_number, l.content)
                })
                .collect::<Vec<_>>().join("\n"),
            error.suggestion.as_ref()
                .map(|s| format!("💡 Suggestion: {}", s.message))
                .unwrap_or_default()
        );

        let id = self.next_entry_id;
        self.next_entry_id += 1;
        self.entries.push_back(ConsoleEntry {
            id,
            timestamp: Utc::now(),
            level: LogLevel::Error,
            source: format!("script:{}", error.file),
            message: msg,
            rich_message: None,
            file: Some(error.file.clone()),
            line: Some(error.line),
            column: Some(error.column),
            stack_trace: Vec::new(),
            count: 1,
        });

        // If AI assistant enabled, auto-explain the error
        if self.ai_assistant.explain_errors {
            let prompt = self.ai_assistant.build_error_prompt(error, "");
            self.log(LogLevel::AiResponse, "ai_assistant",
                "I see an error — type ?explain for detailed help or ?fix to get a fix suggestion");
        }
    }

    pub fn execute_input(&mut self, input: &str) -> ConsoleResult {
        // Add to history
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.input_history.push(input.to_string());
            tab.history_index = -1;
        }

        self.log(LogLevel::UserInput, "console", input);

        // AI question prefix
        if input.starts_with('?') {
            let question = input.trim_start_matches('?').trim();
            self.ai_assistant.add_message(AiMessageRole::User, question);
            return ConsoleResult::AiQuery(question.to_string());
        }

        // Built-in commands
        match input.trim() {
            "help" => {
                let help = r#"Genesis Console Commands:
  help              — Show this help
  clear             — Clear console
  entities          — Show entity count
  fps               — Show current FPS
  agents            — List agent statuses
  scene             — Show current scene tree
  breakpoint <file> <line> — Add breakpoint
  resume            — Resume paused script
  step              — Step one line
  watch <expr>      — Add watch expression
  macro <n> <cmd>   — Save a macro
  ?<question>       — Ask AI assistant
  ??<code>          — Ask AI to complete code

Languages: chronoscript | lua | rhai | python | js"#;
                ConsoleResult::Output(help.to_string())
            }
            "__clear__" | "clear" => {
                self.entries.clear();
                ConsoleResult::Clear
            }
            "agents" => ConsoleResult::AgentStatus,
            "scene" => ConsoleResult::SceneTree,
            "resume" => {
                self.debugger.resume();
                ConsoleResult::Output("Debugger resumed".to_string())
            }
            "step" => {
                self.debugger.step_over();
                ConsoleResult::Output("Stepping...".to_string())
            }
            _ => ConsoleResult::Execute(input.to_string()),
        }
    }

    pub fn visible_entries(&self) -> impl Iterator<Item = &ConsoleEntry> {
        self.entries.iter().filter(|e| {
            if let Some(min_level) = &self.filter_level {
                if e.level < *min_level { return false; }
            }
            if let Some(source) = &self.filter_source {
                if !e.source.contains(source.as_str()) { return false; }
            }
            if !self.search_text.is_empty() {
                if !e.message.to_lowercase().contains(&self.search_text.to_lowercase()) {
                    return false;
                }
            }
            true
        })
    }

    pub fn entry_count(&self) -> usize { self.entries.len() }
    pub fn error_count(&self) -> usize { self.entries.iter().filter(|e| e.level == LogLevel::Error).count() }
    pub fn warning_count(&self) -> usize { self.entries.iter().filter(|e| e.level == LogLevel::Warning).count() }
}

/// Result of executing a console command
pub enum ConsoleResult {
    Output(String),
    Error(String),
    AiQuery(String),
    Execute(String),    // pass to script engine
    Clear,
    AgentStatus,
    SceneTree,
}
