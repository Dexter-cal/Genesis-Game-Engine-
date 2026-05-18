//! ChronoVerse 2D Engine
//!
//! Full 2D game support alongside 3D — not an afterthought.
//! Like Godot, we support BOTH 2D and 3D in the same engine.
//! You can even mix them (2D UI over 3D world, 3D elements in 2D scene).
//!
//! 2D Features:
//! - Sprite rendering (animated, atlas, 9-slice)
//! - Tilemaps (isometric, hexagonal, orthographic)
//! - 2D Physics (Box2D-style via Rapier2D)
//! - 2D Lighting (normal maps, point lights, shadows)
//! - 2D Particles
//! - 2D Camera (zoom, pan, shake, split-screen)
//! - 2D Pathfinding (navmesh, A* on grid)
//! - 2D Parallax layers
//! - 2D Collision shapes
//! - Spine / DragonBones skeletal animation
//! - 2D Shaders (pixel art, CRT, chromatic aberration)
//! - Text rendering (see genesis-text)
//! - 2D Audio (positional in 2D space)
//!
//! Node system (like Godot):
//! Every 2D object is a NODE. Nodes have:
//! - A parent node (forms a tree)
//! - Transform (position, rotation, scale)
//! - A type (Sprite2D, Body2D, Area2D, Camera2D, etc.)
//! - Scripts attached
//! - Signals they emit
//! - Properties (edited in Inspector)
//!
//! This node system works in BOTH 2D and 3D.

pub mod node;
pub mod sprite;
pub mod tilemap;
pub mod physics2d;
pub mod camera2d;
pub mod light2d;
pub mod particle2d;
pub mod nav2d;
pub mod scene2d;

pub use node::*;
pub use sprite::*;
pub use tilemap::*;
pub use camera2d::*;
pub use scene2d::*;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use genesis_math::{vec2::Vec2, color::Color};

// ═══════════════════════════════════════════════════════════════════════════
// NODE SYSTEM — The foundation of both 2D and 3D scene graphs
// Like Godot's nodes, but everything is Rust-native
// ═══════════════════════════════════════════════════════════════════════════

/// Every object in ChronoVerse is a Node.
/// Nodes form a scene tree. Children inherit parent transforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub visible: bool,
    pub active: bool,
    pub transform: NodeTransform,
    pub scripts: Vec<ScriptRef>,
    pub signals: HashMap<String, Vec<SignalConnection>>,
    pub groups: Vec<String>,
    pub meta: HashMap<String, serde_json::Value>, // user-defined metadata
    pub editor_only: bool,     // only exists in editor, stripped at export
    pub tool_mode: bool,       // script runs in editor too
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTransform {
    pub position: [f32; 3],    // xyz for 3D, xy+z(layer) for 2D
    pub rotation: [f32; 4],    // quaternion for 3D, [0,0,sin,cos] for 2D
    pub scale: [f32; 3],
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

/// All node types in ChronoVerse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    // ─── ROOT / CONTAINERS ────────────────────────────────────────────────
    /// Root scene node
    Scene { scene_file: String },
    /// Empty container for organizing
    Node2D,
    Node3D,
    /// Instantiated sub-scene
    SceneInstance { scene_path: String },

    // ─── 2D VISUAL ────────────────────────────────────────────────────────
    Sprite2D { texture: String, atlas_region: Option<[f32; 4]>, centered: bool, flip_h: bool, flip_v: bool },
    AnimatedSprite2D { sprite_frames: String, current_animation: String, playing: bool, speed: f32 },
    NinePatchRect { texture: String, patch_margin: [f32; 4] },
    TileMap { tileset: String, cell_size: [f32; 2], layers: Vec<TileLayer> },
    Polygon2D { points: Vec<[f32; 2]>, color: [f32; 4], texture: Option<String> },
    Line2D { points: Vec<[f32; 2]>, width: f32, color: [f32; 4] },
    MeshInstance2D { mesh: String, material: String },
    MultiMeshInstance2D { mesh: String, material: String, count: u32 },
    Parallax { layer_velocity: [f32; 2], mirroring: [f32; 2] },
    BackBuffer2D,  // captures what's behind for post-processing

    // ─── 3D VISUAL ────────────────────────────────────────────────────────
    MeshInstance3D { mesh: String, materials: Vec<String> },
    MultiMeshInstance3D { mesh: String, material: String, count: u32 },
    SkinnedMeshInstance3D { mesh: String, skeleton: String, materials: Vec<String> },
    Decal { texture: String, size: [f32; 3] },
    GpuParticles3D { process_material: String, draw_pass_mesh: Option<String> },
    CpuParticles3D,
    Label3D { text: String, font: Option<String>, pixel_size: f32 },
    ReflectionProbe { size: [f32; 3], update_mode: String },
    VoxelGI { size: [f32; 3], subdiv: String },
    LightmapGI,
    FogVolume { size: [f32; 3], density: f32, color: [f32; 4] },

    // ─── LIGHTS ───────────────────────────────────────────────────────────
    DirectionalLight3D { color: [f32; 3], energy: f32, shadow: bool },
    OmniLight3D { color: [f32; 3], energy: f32, range: f32, shadow: bool },
    SpotLight3D { color: [f32; 3], energy: f32, range: f32, angle: f32, shadow: bool },
    DirectionalLight2D { color: [f32; 4], energy: f32, shadow: bool },
    PointLight2D { color: [f32; 4], energy: f32, range: f32, shadow: bool },
    LightOccluder2D { occluder_polygon: Vec<[f32; 2]> },

    // ─── CAMERAS ──────────────────────────────────────────────────────────
    Camera2D { zoom: [f32; 2], offset: [f32; 2], limit: [f32; 4], drag_enabled: bool },
    Camera3D { fov: f32, near: f32, far: f32, projection: CameraProjection },
    XrCamera { tracking_origin: String },

    // ─── 2D PHYSICS ───────────────────────────────────────────────────────
    StaticBody2D,
    AnimatableBody2D,
    RigidBody2D { mass: f32, gravity_scale: f32, linear_damp: f32, angular_damp: f32 },
    CharacterBody2D { motion_mode: MotionMode },
    Area2D { gravity_space_override: bool, gravity: f32 },
    CollisionShape2D { shape: Shape2D },
    CollisionPolygon2D { polygon: Vec<[f32; 2]> },
    TileMapLayer { use_kinematic_bodies: bool },
    RayCast2D { target: [f32; 2], enabled: bool },

    // ─── 3D PHYSICS ───────────────────────────────────────────────────────
    StaticBody3D,
    AnimatableBody3D,
    RigidBody3D { mass: f32, gravity_scale: f32 },
    CharacterBody3D,
    Area3D { gravity_override: bool },
    CollisionShape3D { shape: Shape3D },
    CollisionPolygon3D { depth: f32 },
    RayCast3D { target: [f32; 3], enabled: bool, collide_with_areas: bool },
    ShapeCast3D { shape: Shape3D, target: [f32; 3] },
    VehicleBody3D,
    VehicleWheel3D { use_as_traction: bool, use_as_steering: bool },

    // ─── NAVIGATION ───────────────────────────────────────────────────────
    NavigationRegion2D { navigation_polygon: Vec<Vec<[f32; 2]>> },
    NavigationRegion3D { navigation_mesh_source: String },
    NavigationAgent2D { path_desired_distance: f32, target_desired_distance: f32 },
    NavigationAgent3D { path_desired_distance: f32, target_desired_distance: f32, height: f32 },
    NavigationLink2D { start: [f32; 2], end: [f32; 2] },
    NavigationLink3D { start: [f32; 3], end: [f32; 3] },
    NavigationObstacle2D,
    NavigationObstacle3D { radius: f32, height: f32 },

    // ─── ANIMATION ────────────────────────────────────────────────────────
    AnimationPlayer { autoplay: Option<String>, speed_scale: f32 },
    AnimationTree { tree_root: String, active: bool },
    Skeleton3D,
    BoneAttachment3D { bone_name: String },
    SkeletonIK3D { root_bone: String, tip_bone: String, target: [f32; 3] },
    AnimationMixer,

    // ─── AUDIO ────────────────────────────────────────────────────────────
    AudioStreamPlayer { stream: String, volume_db: f32, autoplay: bool, bus: String },
    AudioStreamPlayer2D { stream: String, max_distance: f32, volume_db: f32 },
    AudioStreamPlayer3D { stream: String, max_distance: f32, volume_db: f32, attenuation_model: String },
    AudioListener2D,
    AudioListener3D,

    // ─── UI NODES ────────────────────────────────────────────────────────
    Control { anchor_left: f32, anchor_top: f32, anchor_right: f32, anchor_bottom: f32 },
    Button { text: String, icon: Option<String>, flat: bool },
    Label { text: String, autowrap: bool, horizontal_alignment: String },
    RichTextLabel { bbcode_text: String, fit_content: bool },
    LineEdit { placeholder_text: String, secret: bool, max_length: i32 },
    TextEdit { text: String, syntax_highlighter: Option<String> },
    ProgressBar { min_value: f32, max_value: f32, value: f32, show_percentage: bool },
    HBoxContainer,  VBoxContainer, GridContainer { columns: u32 },
    ScrollContainer { horizontal_scroll_mode: String, vertical_scroll_mode: String },
    TabContainer, PanelContainer, MarginContainer { margin: f32 },
    Popup, PopupMenu, ColorPicker, FileDialog,
    CodeEdit { language: String, font: Option<String> }, // for our console
    TextureRect { texture: String, expand_mode: String, stretch_mode: String },
    TextureButton { texture_normal: Option<String>, texture_pressed: Option<String> },
    NinePatchPanel { texture: String, patch_margin: [f32; 4] },
    VideoStreamPlayer { stream: String, autoplay: bool },
    Slider { min_value: f32, max_value: f32, value: f32, step: f32, vertical: bool },
    SpinBox { min_value: f32, max_value: f32, step: f32 },
    CheckBox { text: String, checked: bool },
    CheckButton { text: String, checked: bool, toggle_mode: bool },
    OptionButton { items: Vec<String>, selected: i32 },
    Tree { columns: u32 },
    ItemList { max_columns: u32 },
    GraphNode { title: String, draggable: bool },
    GraphEdit, // for our visual node editor
    SubViewport { size: [u32; 2], transparent_bg: bool },
    SubViewportContainer,

    // ─── PATH ─────────────────────────────────────────────────────────────
    Path2D { curve: Vec<[f32; 2]> },
    Path3D { curve: Vec<[f32; 3]> },
    PathFollow2D { progress: f32, rotates: bool },
    PathFollow3D { progress: f32, rotates: bool },

    // ─── ENVIRONMENT ──────────────────────────────────────────────────────
    WorldEnvironment { environment: EnvironmentSettings },
    Sky { sky_material: String },
    Sun { direction: [f32; 3], energy: f32, color: [f32; 3] },

    // ─── XR / VR ─────────────────────────────────────────────────────────
    XrOrigin3D,
    XrController3D { tracker: String, pose: String },
    XrAnchor3D,
    XrHandModifier3D { hand: String },
    XrFaceModifier3D,

    // ─── NETWORKING ───────────────────────────────────────────────────────
    MultiplayerSynchronizer { root_path: String, sync_interval: f32 },
    MultiplayerSpawner { spawn_path: String },

    // ─── SPECIAL ─────────────────────────────────────────────────────────
    Marker2D,   // invisible position marker
    Marker3D,
    Timer { wait_time: f32, one_shot: bool, autostart: bool },
    Tween,
    ShaderMaterial2D { shader: String },
    VisibleOnScreenNotifier2D { rect: [f32; 4] },
    VisibleOnScreenNotifier3D { aabb: [f32; 6] },
    RemoteTransform2D { remote_path: String },
    RemoteTransform3D { remote_path: String },
    Curve2D { bake_interval: f32 },
    Curve3D { bake_interval: f32 },
    CsgBox { size: [f32; 3], material: Option<String> },         // CSG for prototyping
    CsgSphere { radius: f32, material: Option<String> },
    CsgCylinder { radius: f32, height: f32, material: Option<String> },
    CsgCombiner { operation: String },   // union/subtraction/intersection
    OccluderInstance3D { occluder: String },

    // ─── CHRONOVERSE SPECIFIC ─────────────────────────────────────────────
    /// An AI agent attached to this node
    AgentNode { agent_type: String, config: serde_json::Value },
    /// Visual Script Node (our Blueprint equivalent)
    VisualScriptNode { graph_id: String },
    /// Real-time cinematic viewport
    CinematicViewport { camera_id: String, post_process: Vec<String> },
    /// Water surface node
    WaterSurface { ocean_config_id: String },
    /// Terrain node
    Terrain3D { config_id: String, chunk_count: [i32; 2] },
    /// Interior room node
    Room { room_definition_id: String },
    Custom { type_name: String, data: serde_json::Value },
}

/// 2D collision shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape2D {
    Circle { radius: f32 },
    Rectangle { size: [f32; 2] },
    Capsule { radius: f32, height: f32 },
    Polygon { points: Vec<[f32; 2]> },
    ConvexPolygon { points: Vec<[f32; 2]> },
    Segment { a: [f32; 2], b: [f32; 2] },
}

/// 3D collision shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape3D {
    Sphere { radius: f32 },
    Box { size: [f32; 3] },
    Capsule { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
    ConvexHull { mesh: String },
    ConcaveMesh { mesh: String },
    HeightMap { map: String, size: [f32; 2] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraProjection { Perspective, Orthogonal { size: f32 }, Frustum { offset: [f32; 2] } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MotionMode { Grounded, Floating }

/// Script reference attached to a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRef {
    pub script_id: String,
    pub path: String,
    pub language: String,
    pub enabled: bool,
}

/// A signal connection (emit from one node, call on another)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConnection {
    pub signal_name: String,
    pub target_node: String,
    pub target_method: String,
    pub binds: Vec<serde_json::Value>,
    pub flags: u32,
    pub deferred: bool,
    pub one_shot: bool,
}

/// Environment/sky/fog settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSettings {
    pub background: BackgroundMode,
    pub sky: Option<String>,
    pub ambient_color: [f32; 4],
    pub ambient_energy: f32,
    pub fog_enabled: bool,
    pub fog_color: [f32; 4],
    pub fog_density: f32,
    pub glow_enabled: bool,
    pub ssao_enabled: bool,
    pub ssil_enabled: bool,
    pub sdfgi_enabled: bool,
    pub volumetric_fog: bool,
    pub tonemap_mode: TonemapMode,
    pub tonemap_exposure: f32,
    pub tonemap_white: f32,
    pub brightness: f32, pub contrast: f32, pub saturation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundMode { Sky, Color, Canvas, KeepPrev, CameraFeed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TonemapMode { Linear, Reinhardt, Filmic, Aces }

// ─── SCENE TREE ──────────────────────────────────────────────────────────────

/// The complete scene tree — nodes arranged hierarchically
pub struct SceneTree {
    pub nodes: HashMap<String, Node>,
    pub root_id: Option<String>,
    pub running: bool,
    pub paused: bool,
    pub physics_fps: u32,
    pub process_fps: u32,
    pub current_scene: Option<String>,
    pub auto_accept_quit: bool,
    /// Groups for fast node lookup
    pub groups: HashMap<String, Vec<String>>,
}

impl SceneTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: None,
            running: false,
            paused: false,
            physics_fps: 60,
            process_fps: 60,
            current_scene: None,
            auto_accept_quit: true,
            groups: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, mut node: Node, parent_id: Option<&str>) -> String {
        let id = node.id.clone();
        node.parent_id = parent_id.map(|s| s.to_string());

        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(pid) {
                parent.children.push(id.clone());
            }
        } else if self.root_id.is_none() {
            self.root_id = Some(id.clone());
        }

        // Register groups
        for group in &node.groups.clone() {
            self.groups.entry(group.clone()).or_default().push(id.clone());
        }

        self.nodes.insert(id.clone(), node);
        id
    }

    pub fn remove_node(&mut self, id: &str) {
        // Remove from parent
        if let Some(node) = self.nodes.get(id) {
            if let Some(pid) = node.parent_id.clone() {
                if let Some(parent) = self.nodes.get_mut(&pid) {
                    parent.children.retain(|c| c != id);
                }
            }
        }
        // Remove from groups
        for group_members in self.groups.values_mut() {
            group_members.retain(|m| m != id);
        }
        // Remove node and all children
        let children: Vec<String> = self.nodes.get(id)
            .map(|n| n.children.clone()).unwrap_or_default();
        for child_id in children { self.remove_node(&child_id); }
        self.nodes.remove(id);
    }

    pub fn get_nodes_in_group(&self, group: &str) -> Vec<&Node> {
        self.groups.get(group)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn get_node_by_path(&self, path: &str) -> Option<&Node> {
        // Path like "Root/Player/Camera"
        let parts: Vec<&str> = path.split('/').collect();
        let mut current_id = self.root_id.as_deref()?;
        for part in &parts[1..] {
            let node = self.nodes.get(current_id)?;
            current_id = node.children.iter()
                .find(|cid| self.nodes.get(*cid).map(|n| n.name == *part).unwrap_or(false))?;
        }
        self.nodes.get(current_id)
    }

    pub fn emit_signal(&self, from_node_id: &str, signal: &str) -> Vec<(&str, &str)> {
        let node = match self.nodes.get(from_node_id) { Some(n) => n, None => return vec![] };
        let connections = match node.signals.get(signal) { Some(c) => c, None => return vec![] };
        connections.iter().map(|c| (c.target_node.as_str(), c.target_method.as_str())).collect()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn is_running(&self) -> bool { self.running }
}

// ─── SPRITE SYSTEM ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteFrames {
    pub id: String,
    pub animations: HashMap<String, SpriteAnimation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAnimation {
    pub name: String,
    pub fps: f32,
    pub looping: bool,
    pub frames: Vec<SpriteFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub texture: String,
    pub region: Option<[f32; 4]>,  // atlas region
    pub duration: f32,             // frame duration multiplier
}

// ─── TILEMAP ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayer {
    pub name: String,
    pub tiles: HashMap<[i32; 2], TileCell>,
    pub z_index: i32,
    pub visible: bool,
    pub modulate: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileCell {
    pub tileset_id: String,
    pub tile_id: u32,
    pub rotation: u8,
    pub flip_h: bool,
    pub flip_v: bool,
    pub transpose: bool,
    pub terrain: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSet {
    pub id: String,
    pub name: String,
    pub tile_size: [u32; 2],
    pub tiles: HashMap<u32, TileData>,
    pub terrain_sets: Vec<TerrainSet>,
    pub custom_data_layers: Vec<String>,
    pub tile_shape: TileShape,
    pub tile_layout: TileLayout,
    pub tile_offset_axis: TileOffsetAxis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileData {
    pub id: u32,
    pub texture_region: [f32; 4],
    pub texture: String,
    pub collision_polygons: Vec<Vec<[f32; 2]>>,
    pub physics_layer: u32,
    pub terrain: Option<u32>,
    pub navigation_polygon: Vec<[f32; 2]>,
    pub occluder_polygon: Vec<[f32; 2]>,
    pub custom_data: HashMap<String, serde_json::Value>,
    pub probability: f32,
    pub animation: Option<SpriteAnimation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainSet {
    pub name: String,
    pub mode: TerrainMode,
    pub terrains: Vec<Terrain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub name: String,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileShape { Square, Isometric, HalfOffsetSquare, Hexagon }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileLayout { TiledGridLayout, Isometric, Stairs, Diamond }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileOffsetAxis { Horizontal, Vertical }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainMode { MatchCornersAndSides, MatchCorners, MatchSides }

// ─── CAMERA 2D ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera2DState {
    pub position: Vec2,
    pub zoom: Vec2,
    pub rotation: f32,
    pub offset: Vec2,
    pub limit: CameraLimit,
    pub drag: CameraDrag,
    pub shake_intensity: f32,
    pub shake_timer: f32,
    pub trauma: f32,     // 0-1 current trauma (produces shake)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraLimit {
    pub left: f32, pub right: f32, pub top: f32, pub bottom: f32,
    pub enabled: bool,
    pub draw_limit: bool,
    pub smoothing_enabled: bool,
    pub smoothing_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraDrag {
    pub enabled: bool,
    pub horizontal_drag_margin: f32,
    pub vertical_drag_margin: f32,
    pub horizontal_offset: f32,
    pub vertical_offset: f32,
}

impl Camera2DState {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            offset: Vec2::ZERO,
            limit: CameraLimit {
                left: -1e7, right: 1e7, top: -1e7, bottom: 1e7,
                enabled: false, draw_limit: false,
                smoothing_enabled: false, smoothing_speed: 5.0,
            },
            drag: CameraDrag {
                enabled: false,
                horizontal_drag_margin: 0.2, vertical_drag_margin: 0.2,
                horizontal_offset: 0.0, vertical_offset: 0.0,
            },
            shake_intensity: 0.0,
            shake_timer: 0.0,
            trauma: 0.0,
        }
    }

    /// Add trauma for camera shake (trauma decays, shake = trauma²)
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    pub fn tick(&mut self, delta: f32) {
        // Decay trauma
        self.trauma = (self.trauma - delta * 1.5).max(0.0);
        // Apply shake based on trauma²
        self.shake_intensity = self.trauma * self.trauma;
    }

    pub fn pixel_zoom(&self, viewport_size: Vec2) -> Vec2 {
        Vec2::new(viewport_size.x * self.zoom.x, viewport_size.y * self.zoom.y)
    }
}

// ─── SCENE 2D ─────────────────────────────────────────────────────────────────

/// A complete 2D scene file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene2D {
    pub id: String,
    pub name: String,
    pub path: String,
    pub tree: Vec<NodeDef>,   // serialized node definitions
    pub resources: Vec<ResourceRef>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    pub name: String,
    pub node_type: String,
    pub parent: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub scripts: Vec<String>,
    pub children: Vec<NodeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub path: String,
    pub resource_type: String,
    pub uid: String,
}

// Stubs for nav/light/physics
pub mod nav2d { pub struct NavGrid; }
pub mod light2d { pub struct Light2DRenderer; }
pub mod physics2d { pub struct PhysicsWorld2D; }
pub mod particle2d { pub struct ParticleEmitter2D; }
pub mod scene2d { pub use super::{Scene2D, NodeDef}; }
pub mod sprite { pub use super::{SpriteFrames, SpriteAnimation, SpriteFrame}; }
pub mod tilemap { pub use super::{TileSet, TileData, TileLayer, TileCell, TileShape}; }
pub mod camera2d { pub use super::{Camera2DState, CameraLimit, CameraDrag}; }
pub mod node { pub use super::{Node, NodeType, NodeTransform, ScriptRef, SignalConnection, SceneTree}; }
