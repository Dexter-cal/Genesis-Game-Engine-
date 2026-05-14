//! Genesis Engine Editor UI System
//!
//! Complete professional editor interface:
//!
//! MAIN LAYOUT:
//! ┌─────────────────────────────────────────────────────────┐
//! │ Menu Bar: File Edit View Project Build Debug Help       │
//! ├──────────┬──────────────────────────────┬──────────────┤
//! │          │                              │              │
//! │  Scene   │      3D Viewport             │  Inspector   │
//! │  Tree    │      (main canvas)           │  Properties  │
//! │          │                              │              │
//! │          ├──────────────────────────────┤              │
//! │          │  Timeline / Animation        │              │
//! ├──────────┴──────────────────────────────┴──────────────┤
//! │  Asset Browser  │  Console  │  Output  │  AI Chat      │
//! └─────────────────────────────────────────────────────────┘
//!
//! PANELS:
//! - Scene Tree (node hierarchy)
//! - 3D/2D Viewport (main editing canvas, gizmos)
//! - Inspector (properties of selected node)
//! - Asset Browser (files, search, preview)
//! - Console (REPL, logs, AI assistant)
//! - Timeline (animation, cutscene editor)
//! - Material Editor (node-based shader graph)
//! - Script Editor (syntax highlighting, AI)
//! - Project Settings (engine config)
//! - Build & Export Panel
//! - Performance Profiler
//! - Physics Debug
//! - AI Agents Panel (see what agents are doing)
//! - Studio Panel (virtual production)
//! - Model Inspector (3D modeler)
//!
//! THEMES:
//! - Dark (default, easy on eyes)
//! - Light (for bright environments)
//! - High Contrast (accessibility)
//! - Custom (user-defined colors)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ─── Editor Layout ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorLayout {
    pub name: String,
    pub panels: Vec<PanelLayout>,
    pub dividers: Vec<Divider>,
    pub active_panel: String,
    pub theme: EditorTheme,
    pub font_size: f32,
    pub icon_size: f32,
    pub density: UiDensity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayout {
    pub id: String,
    pub panel_type: PanelType,
    pub position: PanelPosition,
    pub rect: [f32; 4],    // x, y, width, height (normalized 0-1)
    pub visible: bool,
    pub pinned: bool,
    pub floating: bool,
    pub float_rect: Option<[f32; 4]>,
    pub tab_group: Option<String>,
    pub tab_index: u32,
    pub min_size: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelPosition {
    Left, Right, Top, Bottom, Center, Floating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    // Core panels
    SceneTree,
    Viewport3D     { camera_mode: String },
    Viewport2D,
    Inspector,
    AssetBrowser   { path: String },
    Console,
    Timeline,
    Output,

    // Creative panels
    MaterialEditor { material_id: Option<String> },
    ScriptEditor   { file: Option<String> },
    ShaderGraph    { shader_id: Option<String> },
    NodeGraph      { graph_id: Option<String> },
    AnimationGraph,

    // AI panels
    AiChat,
    AgentMonitor,
    AiModeler,
    GpuLab,

    // Production
    StudioPanel,
    VfxCompositor,
    AudioMixer,
    Marketplace,
    AssetGallery,

    // Dev tools
    Profiler,
    PhysicsDebug,
    NetworkMonitor,
    MemoryProfiler,

    // Settings
    ProjectSettings,
    EngineSettings,
    BuildExport,

    // Custom
    Custom { name: String },
}

impl PanelType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SceneTree => "Scene Tree",
            Self::Viewport3D { .. } => "3D Viewport",
            Self::Viewport2D => "2D Viewport",
            Self::Inspector => "Inspector",
            Self::AssetBrowser { .. } => "Asset Browser",
            Self::Console => "Console",
            Self::Timeline => "Timeline",
            Self::Output => "Output",
            Self::MaterialEditor { .. } => "Material Editor",
            Self::ScriptEditor { .. } => "Script Editor",
            Self::ShaderGraph { .. } => "Shader Graph",
            Self::NodeGraph { .. } => "Node Graph",
            Self::AnimationGraph => "Animation Graph",
            Self::AiChat => "AI Assistant",
            Self::AgentMonitor => "Agent Monitor",
            Self::AiModeler => "AI Modeler",
            Self::GpuLab => "GPU Lab",
            Self::StudioPanel => "Studio",
            Self::VfxCompositor => "VFX Compositor",
            Self::AudioMixer => "Audio Mixer",
            Self::Marketplace => "Marketplace",
            Self::AssetGallery => "3D Asset Gallery",
            Self::Profiler => "Profiler",
            Self::PhysicsDebug => "Physics Debug",
            Self::NetworkMonitor => "Network",
            Self::MemoryProfiler => "Memory",
            Self::ProjectSettings => "Project Settings",
            Self::EngineSettings => "Engine Settings",
            Self::BuildExport => "Build & Export",
            Self::Custom { name } => "Custom",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::SceneTree => "🌳",
            Self::Viewport3D { .. } => "🎮",
            Self::Viewport2D => "📐",
            Self::Inspector => "🔍",
            Self::AssetBrowser { .. } => "📁",
            Self::Console => "💻",
            Self::Timeline => "📽️",
            Self::MaterialEditor { .. } => "🎨",
            Self::ScriptEditor { .. } => "📝",
            Self::AiChat => "🤖",
            Self::AgentMonitor => "🕵️",
            Self::Marketplace => "🛒",
            Self::AssetGallery => "💎",
            Self::StudioPanel => "🎬",
            Self::Profiler => "📊",
            Self::BuildExport => "📦",
            _ => "🔧",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divider {
    pub id: String,
    pub direction: DividerDirection,
    pub position: f32,      // 0-1 normalized
    pub min_position: f32,
    pub max_position: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DividerDirection { Horizontal, Vertical }

// ─── Themes ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTheme {
    pub name: String,
    pub colors: ThemeColors,
    pub border_radius: f32,
    pub shadow_strength: f32,
    pub animation_speed: f32,
    pub font: String,
    pub mono_font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background:          [f32; 4],   // main background
    pub surface:             [f32; 4],   // panels, cards
    pub surface_elevated:    [f32; 4],   // elevated panels, dropdowns
    pub surface_sunken:      [f32; 4],   // input fields, tree items
    pub border:              [f32; 4],
    pub primary:             [f32; 4],   // accent / brand color
    pub primary_hover:       [f32; 4],
    pub primary_active:      [f32; 4],
    pub secondary:           [f32; 4],
    pub text:                [f32; 4],
    pub text_secondary:      [f32; 4],
    pub text_disabled:       [f32; 4],
    pub text_on_primary:     [f32; 4],
    pub error:               [f32; 4],
    pub warning:             [f32; 4],
    pub success:             [f32; 4],
    pub info:                [f32; 4],
    pub selection:           [f32; 4],
    pub selection_text:      [f32; 4],
    pub scrollbar:           [f32; 4],
    pub scrollbar_hover:     [f32; 4],
    pub drag_highlight:      [f32; 4],
    pub gizmo_x:             [f32; 4],   // X axis = red
    pub gizmo_y:             [f32; 4],   // Y axis = green
    pub gizmo_z:             [f32; 4],   // Z axis = blue
    pub grid:                [f32; 4],
    pub selection_box:       [f32; 4],
}

impl EditorTheme {
    pub fn dark() -> Self {
        Self {
            name: "Dark (Default)".to_string(),
            colors: ThemeColors {
                background:       [0.098, 0.098, 0.106, 1.0],  // #191919
                surface:          [0.137, 0.137, 0.149, 1.0],  // #232326
                surface_elevated: [0.176, 0.176, 0.196, 1.0],  // #2D2D32
                surface_sunken:   [0.071, 0.071, 0.078, 1.0],  // #121214
                border:           [0.227, 0.227, 0.251, 1.0],
                primary:          [0.251, 0.706, 1.0, 1.0],    // #40B4FF (Genesis Neon Blue)
                primary_hover:    [0.376, 0.769, 1.0, 1.0],
                primary_active:   [0.196, 0.627, 0.941, 1.0],
                secondary:        [0.6, 0.35, 1.0, 1.0],       // purple accent
                text:             [0.922, 0.922, 0.937, 1.0],
                text_secondary:   [0.620, 0.620, 0.651, 1.0],
                text_disabled:    [0.380, 0.380, 0.400, 1.0],
                text_on_primary:  [1.0, 1.0, 1.0, 1.0],
                error:            [0.961, 0.361, 0.361, 1.0],  // red
                warning:          [0.984, 0.722, 0.2, 1.0],    // amber
                success:          [0.302, 0.800, 0.502, 1.0],  // green
                info:             [0.251, 0.584, 1.0, 1.0],    // blue
                selection:        [0.251, 0.584, 1.0, 0.3],
                selection_text:   [1.0, 1.0, 1.0, 1.0],
                scrollbar:        [0.300, 0.300, 0.330, 1.0],
                scrollbar_hover:  [0.450, 0.450, 0.490, 1.0],
                drag_highlight:   [0.251, 0.584, 1.0, 0.5],
                gizmo_x:          [0.9, 0.2, 0.2, 1.0],
                gizmo_y:          [0.2, 0.8, 0.2, 1.0],
                gizmo_z:          [0.2, 0.4, 0.9, 1.0],
                grid:             [0.25, 0.25, 0.28, 0.5],
                selection_box:    [0.251, 0.584, 1.0, 0.2],
            },
            border_radius: 6.0,
            shadow_strength: 0.4,
            animation_speed: 1.0,
            font: "Inter".to_string(),
            mono_font: "JetBrains Mono".to_string(),
        }
    }

    pub fn light() -> Self {
        let mut t = Self::dark();
        t.name = "Light".to_string();
        t.colors.background       = [0.95, 0.95, 0.96, 1.0];
        t.colors.surface          = [0.99, 0.99, 1.00, 1.0];
        t.colors.surface_elevated = [1.00, 1.00, 1.00, 1.0];
        t.colors.surface_sunken   = [0.91, 0.91, 0.93, 1.0];
        t.colors.text             = [0.08, 0.08, 0.10, 1.0];
        t.colors.text_secondary   = [0.40, 0.40, 0.45, 1.0];
        t.colors.border           = [0.82, 0.82, 0.85, 1.0];
        t.shadow_strength = 0.1;
        t
    }

    pub fn high_contrast() -> Self {
        let mut t = Self::dark();
        t.name = "High Contrast".to_string();
        t.colors.background       = [0.0, 0.0, 0.0, 1.0];
        t.colors.surface          = [0.08, 0.08, 0.08, 1.0];
        t.colors.text             = [1.0, 1.0, 1.0, 1.0];
        t.colors.border           = [1.0, 1.0, 1.0, 0.5];
        t.colors.primary          = [0.0, 0.8, 1.0, 1.0];
        t
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiDensity { Compact, Normal, Comfortable }

// ─── Menu System ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuBar {
    pub menus: Vec<Menu>,
    pub breadcrumb: Vec<String>,
    pub search_visible: bool,
    pub notifications: Vec<EditorNotification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub label: String,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: String,
    pub separator_after: bool,
    pub enabled: bool,
    pub checked: bool,
    pub submenu: Vec<MenuItem>,
    pub icon: Option<String>,
}

impl MenuBar {
    pub fn default_genesis() -> Self {
        Self {
            menus: vec![
                Menu { label: "File".to_string(), items: vec![
                    MenuItem { label: "New Project...".to_string(), shortcut: Some("Ctrl+Shift+N".to_string()), action: "file.new_project".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("📄".to_string()) },
                    MenuItem { label: "Open Project...".to_string(), shortcut: Some("Ctrl+O".to_string()), action: "file.open_project".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("📁".to_string()) },
                    MenuItem { label: "Save Scene".to_string(), shortcut: Some("Ctrl+S".to_string()), action: "file.save_scene".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("💾".to_string()) },
                    MenuItem { label: "Save All".to_string(), shortcut: Some("Ctrl+Shift+S".to_string()), action: "file.save_all".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: None },
                    MenuItem { label: "Build & Export...".to_string(), shortcut: Some("Ctrl+B".to_string()), action: "file.build".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("📦".to_string()) },
                    MenuItem { label: "Quit".to_string(), shortcut: Some("Ctrl+Q".to_string()), action: "file.quit".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                ]},
                Menu { label: "Edit".to_string(), items: vec![
                    MenuItem { label: "Undo".to_string(), shortcut: Some("Ctrl+Z".to_string()), action: "edit.undo".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                    MenuItem { label: "Redo".to_string(), shortcut: Some("Ctrl+Y".to_string()), action: "edit.redo".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: None },
                    MenuItem { label: "Cut".to_string(), shortcut: Some("Ctrl+X".to_string()), action: "edit.cut".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                    MenuItem { label: "Copy".to_string(), shortcut: Some("Ctrl+C".to_string()), action: "edit.copy".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                    MenuItem { label: "Paste".to_string(), shortcut: Some("Ctrl+V".to_string()), action: "edit.paste".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: None },
                    MenuItem { label: "Project Settings...".to_string(), shortcut: None, action: "edit.project_settings".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("⚙️".to_string()) },
                    MenuItem { label: "Engine Settings...".to_string(), shortcut: None, action: "edit.engine_settings".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                ]},
                Menu { label: "View".to_string(), items: vec![
                    MenuItem { label: "Scene Tree".to_string(), shortcut: Some("Ctrl+1".to_string()), action: "view.toggle_scene_tree".to_string(), separator_after: false, enabled: true, checked: true, submenu: vec![], icon: Some("🌳".to_string()) },
                    MenuItem { label: "Inspector".to_string(), shortcut: Some("Ctrl+2".to_string()), action: "view.toggle_inspector".to_string(), separator_after: false, enabled: true, checked: true, submenu: vec![], icon: Some("🔍".to_string()) },
                    MenuItem { label: "Asset Browser".to_string(), shortcut: Some("Ctrl+3".to_string()), action: "view.toggle_assets".to_string(), separator_after: false, enabled: true, checked: true, submenu: vec![], icon: Some("📁".to_string()) },
                    MenuItem { label: "Console".to_string(), shortcut: Some("Ctrl+`".to_string()), action: "view.toggle_console".to_string(), separator_after: true, enabled: true, checked: true, submenu: vec![], icon: Some("💻".to_string()) },
                    MenuItem { label: "AI Assistant".to_string(), shortcut: Some("Ctrl+Shift+A".to_string()), action: "view.toggle_ai".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("🤖".to_string()) },
                    MenuItem { label: "Theme".to_string(), shortcut: None, action: "".to_string(), separator_after: false, enabled: true, checked: false, icon: None,
                        submenu: vec![
                            MenuItem { label: "Dark (Default)".to_string(), shortcut: None, action: "view.theme.dark".to_string(), separator_after: false, enabled: true, checked: true, submenu: vec![], icon: None },
                            MenuItem { label: "Light".to_string(), shortcut: None, action: "view.theme.light".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                            MenuItem { label: "High Contrast".to_string(), shortcut: None, action: "view.theme.contrast".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                        ],
                    },
                ]},
                Menu { label: "Add".to_string(), items: vec![
                    MenuItem { label: "3D Mesh...".to_string(), shortcut: None, action: "add.mesh".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("📦".to_string()) },
                    MenuItem { label: "Light".to_string(), shortcut: None, action: "".to_string(), separator_after: false, enabled: true, checked: false, icon: Some("💡".to_string()), submenu: vec![
                        MenuItem { label: "Directional Light".to_string(), shortcut: None, action: "add.directional_light".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                        MenuItem { label: "Point Light".to_string(), shortcut: None, action: "add.point_light".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                        MenuItem { label: "Spot Light".to_string(), shortcut: None, action: "add.spot_light".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: None },
                    ]},
                    MenuItem { label: "Camera".to_string(), shortcut: None, action: "add.camera".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("📷".to_string()) },
                    MenuItem { label: "NPC...".to_string(), shortcut: None, action: "add.npc".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🧑".to_string()) },
                    MenuItem { label: "AI Agent Node...".to_string(), shortcut: None, action: "add.agent_node".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🤖".to_string()) },
                    MenuItem { label: "Water Surface".to_string(), shortcut: None, action: "add.water".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🌊".to_string()) },
                    MenuItem { label: "Terrain".to_string(), shortcut: None, action: "add.terrain".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("⛰️".to_string()) },
                ]},
                Menu { label: "AI".to_string(), items: vec![
                    MenuItem { label: "Ask AI to Build...".to_string(), shortcut: Some("Ctrl+Shift+B".to_string()), action: "ai.build_prompt".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("✨".to_string()) },
                    MenuItem { label: "Generate Scene...".to_string(), shortcut: None, action: "ai.generate_scene".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🌍".to_string()) },
                    MenuItem { label: "Generate NPC...".to_string(), shortcut: None, action: "ai.generate_npc".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🧑".to_string()) },
                    MenuItem { label: "Generate Assets...".to_string(), shortcut: None, action: "ai.generate_assets".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("🎨".to_string()) },
                    MenuItem { label: "Agent Monitor".to_string(), shortcut: None, action: "ai.agent_monitor".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🕵️".to_string()) },
                    MenuItem { label: "AI Settings...".to_string(), shortcut: None, action: "ai.settings".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("⚙️".to_string()) },
                ]},
                Menu { label: "Studio".to_string(), items: vec![
                    MenuItem { label: "Open Studio".to_string(), shortcut: Some("Ctrl+Shift+T".to_string()), action: "studio.open".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🎬".to_string()) },
                    MenuItem { label: "New Recording Session".to_string(), shortcut: None, action: "studio.new_session".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🔴".to_string()) },
                    MenuItem { label: "Open 3D Modeler".to_string(), shortcut: Some("Ctrl+Shift+M".to_string()), action: "studio.modeler".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("🗿".to_string()) },
                    MenuItem { label: "VFX Compositor".to_string(), shortcut: None, action: "studio.compositor".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🌟".to_string()) },
                ]},
                Menu { label: "Debug".to_string(), items: vec![
                    MenuItem { label: "Play".to_string(), shortcut: Some("F5".to_string()), action: "debug.play".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("▶️".to_string()) },
                    MenuItem { label: "Pause".to_string(), shortcut: Some("F6".to_string()), action: "debug.pause".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("⏸️".to_string()) },
                    MenuItem { label: "Stop".to_string(), shortcut: Some("F7".to_string()), action: "debug.stop".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("⏹️".to_string()) },
                    MenuItem { label: "Profiler".to_string(), shortcut: None, action: "debug.profiler".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("📊".to_string()) },
                    MenuItem { label: "Physics Debug".to_string(), shortcut: None, action: "debug.physics".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🔭".to_string()) },
                    MenuItem { label: "Developer Console".to_string(), shortcut: Some("Ctrl+Shift+`".to_string()), action: "debug.dev_console".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("🔒".to_string()) },
                ]},
                Menu { label: "Help".to_string(), items: vec![
                    MenuItem { label: "Documentation".to_string(), shortcut: Some("F1".to_string()), action: "help.docs".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("📚".to_string()) },
                    MenuItem { label: "Discord Community".to_string(), shortcut: None, action: "help.discord".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("💬".to_string()) },
                    MenuItem { label: "Report Bug".to_string(), shortcut: None, action: "help.bug".to_string(), separator_after: true, enabled: true, checked: false, submenu: vec![], icon: Some("🐛".to_string()) },
                    MenuItem { label: "About Genesis...".to_string(), shortcut: None, action: "help.about".to_string(), separator_after: false, enabled: true, checked: false, submenu: vec![], icon: Some("ℹ️".to_string()) },
                ]},
            ],
            breadcrumb: Vec::new(),
            search_visible: false,
            notifications: Vec::new(),
        }
    }
}

// ─── Notifications ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorNotification {
    pub id: String,
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action_label: Option<String>,
    pub action: Option<String>,
    pub auto_dismiss_secs: Option<f32>,
    pub elapsed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel { Info, Success, Warning, Error }

// ─── Viewport State ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportState {
    pub camera_position: [f32; 3],
    pub camera_rotation: [f32; 4],
    pub camera_fov: f32,
    pub projection: ViewportProjection,
    pub shading_mode: ViewportShading,
    pub overlay: ViewportOverlay,
    pub gizmo_mode: GizmoMode,
    pub gizmo_space: GizmoSpace,
    pub snap_enabled: bool,
    pub snap_increment: f32,
    pub grid_visible: bool,
    pub grid_size: f32,
    pub stats_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewportProjection { Perspective, Orthographic, Front, Back, Left, Right, Top, Bottom }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewportShading { Wireframe, SolidFlat, SolidSmooth, Material, Rendered }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GizmoMode { Select, Move, Rotate, Scale, Transform }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GizmoSpace { World, Local, View }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportOverlay {
    pub show_normals: bool,
    pub show_wireframe: bool,
    pub show_bounds: bool,
    pub show_colliders: bool,
    pub show_navmesh: bool,
    pub show_lights: bool,
    pub show_cameras: bool,
    pub show_skeletons: bool,
    pub show_pivot: bool,
    pub show_statistics: bool,
}

// ─── Editor State ─────────────────────────────────────────────────────────────

pub struct EditorState {
    pub layout: EditorLayout,
    pub menu: MenuBar,
    pub viewport: ViewportState,
    pub selected_nodes: Vec<String>,
    pub clipboard: Vec<serde_json::Value>,
    pub history: Vec<EditorAction>,
    pub history_index: usize,
    pub project_name: String,
    pub unsaved_changes: bool,
    pub play_mode: PlayMode,
}

#[derive(Debug, Clone)]
pub enum PlayMode { Edit, Playing, Paused }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAction {
    pub name: String,
    pub action_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EditorState {
    pub fn new(project_name: &str) -> Self {
        Self {
            layout: EditorLayout {
                name: "Default".to_string(),
                panels: vec![
                    PanelLayout { id: "scene_tree".to_string(), panel_type: PanelType::SceneTree, position: PanelPosition::Left, rect: [0.0, 0.0, 0.22, 0.7], visible: true, pinned: true, floating: false, float_rect: None, tab_group: None, tab_index: 0, min_size: [200.0, 300.0] },
                    PanelLayout { id: "viewport3d".to_string(), panel_type: PanelType::Viewport3D { camera_mode: "third_far".to_string() }, position: PanelPosition::Center, rect: [0.22, 0.0, 0.54, 0.72], visible: true, pinned: true, floating: false, float_rect: None, tab_group: None, tab_index: 0, min_size: [400.0, 300.0] },
                    PanelLayout { id: "inspector".to_string(), panel_type: PanelType::Inspector, position: PanelPosition::Right, rect: [0.76, 0.0, 0.24, 1.0], visible: true, pinned: true, floating: false, float_rect: None, tab_group: None, tab_index: 0, min_size: [250.0, 400.0] },
                    PanelLayout { id: "assets".to_string(), panel_type: PanelType::AssetBrowser { path: "res://".to_string() }, position: PanelPosition::Bottom, rect: [0.0, 0.72, 0.50, 0.28], visible: true, pinned: false, floating: false, float_rect: None, tab_group: Some("bottom".to_string()), tab_index: 0, min_size: [300.0, 150.0] },
                    PanelLayout { id: "console".to_string(), panel_type: PanelType::Console, position: PanelPosition::Bottom, rect: [0.50, 0.72, 0.26, 0.28], visible: true, pinned: false, floating: false, float_rect: None, tab_group: Some("bottom".to_string()), tab_index: 1, min_size: [300.0, 150.0] },
                ],
                dividers: Vec::new(),
                active_panel: "viewport3d".to_string(),
                theme: EditorTheme::dark(),
                font_size: 14.0,
                icon_size: 18.0,
                density: UiDensity::Normal,
            },
            menu: MenuBar::default_genesis(),
            viewport: ViewportState {
                camera_position: [0.0, 5.0, 10.0],
                camera_rotation: [0.0, 0.0, 0.0, 1.0],
                camera_fov: 75.0,
                projection: ViewportProjection::Perspective,
                shading_mode: ViewportShading::Material,
                overlay: ViewportOverlay {
                    show_normals: false, show_wireframe: false, show_bounds: false,
                    show_colliders: false, show_navmesh: false, show_lights: true,
                    show_cameras: true, show_skeletons: false, show_pivot: true,
                    show_statistics: true,
                },
                gizmo_mode: GizmoMode::Move,
                gizmo_space: GizmoSpace::World,
                snap_enabled: false,
                snap_increment: 0.5,
                grid_visible: true,
                grid_size: 1.0,
                stats_visible: true,
            },
            selected_nodes: Vec::new(),
            clipboard: Vec::new(),
            history: Vec::new(),
            history_index: 0,
            project_name: project_name.to_string(),
            unsaved_changes: false,
            play_mode: PlayMode::Edit,
        }
    }

    pub fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
        }
    }

    pub fn redo(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
        }
    }

    pub fn can_undo(&self) -> bool { self.history_index > 0 }
    pub fn can_redo(&self) -> bool { self.history_index < self.history.len() }
}
