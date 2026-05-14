//! ChronoVerse 3D Modeler — AI-Powered Built-in Blender Alternative
//!
//! A complete 3D modeling studio built into the engine.
//! AI-assisted so even non-artists can create 3D content.
//!
//! Features:
//! - Full polygon modeling (select, extrude, loop cut, bevel, boolean)
//! - Sculpting (like ZBrush — dynamic subdivision, brushes)
//! - Retopology (AI-assisted clean mesh generation)
//! - UV unwrapping (auto-unwrap, seam-based)
//! - Texture painting (paint directly on 3D model)
//! - PBR Material editor (node-based like Blender)
//! - Rigging & skinning (weight painting, auto-weight)
//! - Pose mode (create poses for renders/cutscenes)
//! - Procedural modeling (modifiers, math-driven shapes)
//! - Rendering (quick renders for reference)
//! - LOD generation (auto-reduce polygons)
//! - Format import/export (GLB, FBX, OBJ, USD)
//!
//! AI Agent Integration:
//! - "Model Agent" watches your work and helps
//! - Text→3D model generation (if GPU available)
//! - Image→3D reconstruction (from photos)
//! - Auto-retopo any sculpt
//! - AI texture generation from description
//! - AI animation from reference video
//! - Smart UV unwrapping
//! - Auto-rigging for humanoids

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chronoverse_math::{vec3::Vec3, vec2::Vec2, color::Color};

// ─── Mesh Edit ────────────────────────────────────────────────────────────────

/// An editable mesh (half-edge representation for topology operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMesh {
    pub id: String,
    pub name: String,
    pub vertices: Vec<EditVertex>,
    pub edges: Vec<EditEdge>,
    pub faces: Vec<EditFace>,
    pub uv_layers: Vec<UvLayer>,
    pub vertex_colors: Vec<VertexColorLayer>,
    pub material_slots: Vec<String>,
    pub shape_keys: Vec<ShapeKey>,
    pub modifiers: Vec<Modifier>,
    pub symmetry: SymmetrySettings,
    pub display_settings: DisplaySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub selected: bool,
    pub hidden: bool,
    pub weight: HashMap<String, f32>,   // bone group → weight
    pub crease: f32,
    pub bevel_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditEdge {
    pub v0: u32,
    pub v1: u32,
    pub selected: bool,
    pub hidden: bool,
    pub sharp: bool,
    pub seam: bool,      // UV seam
    pub crease: f32,
    pub bevel_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFace {
    pub vertices: Vec<u32>,   // vertex indices (tri or quad)
    pub material_index: u32,
    pub selected: bool,
    pub hidden: bool,
    pub smooth: bool,         // smooth shading
    pub normal: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvLayer {
    pub name: String,
    pub active: bool,
    pub uvs: Vec<Vec2>,     // one UV per corner (per face vertex)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexColorLayer {
    pub name: String,
    pub colors: Vec<Color>,  // one per corner
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeKey {
    pub name: String,
    pub value: f32,
    pub reference: bool,  // is this the basis key?
    pub positions: Vec<Vec3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymmetrySettings {
    pub enabled: bool,
    pub axis: SymmetryAxis,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymmetryAxis { X, Y, Z, Xy, Xz, Yz, Xyz }

// ─── Modifiers ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub name: String,
    pub enabled: bool,
    pub in_editmode: bool,
    pub render: bool,
    pub kind: ModifierKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifierKind {
    Subsurf        { levels: u32, render_levels: u32, algorithm: SubsurfAlgorithm },
    Solidify       { thickness: f32, fill_rim: bool },
    Bevel          { width: f32, segments: u32, profile: f32, angle_limit: f32 },
    Mirror         { axis: [bool; 3], merge: bool, threshold: f32, bisect: bool },
    Array          { count: u32, offset: [f32; 3], fit_type: ArrayFit },
    Boolean        { operation: BooleanOp, object_id: String, solver: BooleanSolver },
    Displace       { strength: f32, texture_id: Option<String>, direction: DisplaceDir },
    Decimate       { ratio: f32, algorithm: DecimateAlgorithm },
    Wireframe      { thickness: f32, offset: f32 },
    Screw          { angle: f32, steps: u32, axis: u8, screw_offset: f32 },
    Skin           { root_verts: Vec<u32>, use_smooth_shade: bool },
    Remesh         { mode: RemeshMode, octree_depth: u32, scale: f32 },
    SimpleDeform   { mode: SimpleDeformMode, angle: f32, axis: u8 },
    Wave           { height: f32, speed: f32, width: f32, narrowness: f32 },
    Shrinkwrap     { target_id: String, offset: f32, mode: ShrinkwrapMode },
    SurfaceDeform  { target_id: String, strength: f32 },
    Triangulate    { min_vertices: u32, ngon_method: String },
    Weld           { threshold: f32 },
    MeshToVolume   { voxel_size: f32, density: f32 },
    NodeGroup      { node_tree_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubsurfAlgorithm { CatmullClark, Simple }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrayFit { FixedCount, FitLength { length: f32 }, FitCurve { curve_id: String } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BooleanOp { Union, Difference, Intersect }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BooleanSolver { Fast, Exact }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplaceDir { X, Y, Z, Rgb, Xyz, Normal }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecimateAlgorithm { Collapse, UnSubdivide, Planar }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemeshMode { Blocks, Smooth, Sharp, Voxel }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleDeformMode { Twist, Bend, Taper, Stretch }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShrinkwrapMode { Nearest, Projection, NearestVertex, TargetNormal }

// ─── Edit Modes ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditMode {
    Object,         // select/transform objects
    Edit,           // edit mesh vertices/edges/faces
    Sculpt,         // sculpt with brushes
    VertexPaint,    // paint vertex colors
    WeightPaint,    // paint bone weights
    TexturePaint,   // paint textures
    UvEditor,       // UV mapping
    PoseMode,       // animate armature
}

// ─── Sculpt Brushes ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SculptBrush {
    pub name: String,
    pub brush_type: SculptBrushType,
    pub radius: f32,
    pub strength: f32,
    pub hardness: f32,
    pub auto_smooth: f32,
    pub normal_radius: f32,
    pub plane_offset: f32,
    pub plane_trim: f32,
    pub use_frontface: bool,
    pub falloff: FalloffCurve,
    pub stroke_method: StrokeMethod,
    pub texture: Option<String>,
    pub texture_angle: f32,
    pub texture_rake: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SculptBrushType {
    Draw, DrawSharp, Clay, ClayStrips, ClayThumb,
    Layer, Inflate, Blob, Crease, Flatten, Fill, Scrape,
    MultiPlane, Pinch, Grab, ElasticDeform, Snake, Thumb,
    Pose, NudgingSnake, Rotate, Slide, Boundary, Cloth,
    Simplify, Mask, DrawFace, PaintVertex,
    // AI-specific
    AIRefine,   // AI refines selected area to match reference
    AISculpt,   // paint rough area, AI sculpts details
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FalloffCurve { Smooth, Sphere, Root, InvSquare, Sharp, Linear, Constant }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrokeMethod { Dot, Space { spacing: f32 }, Airbrush { rate: f32 }, Anchored, Line, Curve }

// ─── UV Unwrapping ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnwrapMethod {
    /// Conformal (Angle Based) — best for organic
    Conformal,
    /// Least Squares Conformal — even better for organic
    LscmUnwrap,
    /// Smart UV project — good for hard surface
    SmartProject { island_margin: f32, angle_limit: f32 },
    /// Follow active quads — for grid-like surfaces
    FollowActiveQuads,
    /// Lightmap pack — for baking
    LightmapPack { pack_quality: u32, margin: f32 },
    /// Sphere projection
    Sphere,
    /// Cylinder projection
    Cylinder,
    /// Project from view
    ProjectFromView,
    /// AI unwrap — AI finds optimal seams
    AiUnwrap { stretch_target: f32 },
}

// ─── Weight Paint ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightPaintState {
    pub active_group: Option<String>,
    pub brush_weight: f32,
    pub normalize_all: bool,
    pub auto_normalize: bool,
    pub multipaint: bool,
    pub vertex_selection: bool,
    pub zero_weights_action: String,
}

// ─── Modeler Operations (Undo History) ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOperation {
    pub name: String,
    pub op_type: ModelOpType,
    pub affected_verts: Vec<u32>,
    pub before_state: Option<Vec<Vec3>>,
    pub after_state: Option<Vec<Vec3>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelOpType {
    // Mesh editing
    Extrude        { direction: Vec3, amount: f32 },
    Inset          { thickness: f32, depth: f32, individual: bool },
    LoopCut        { ring: u32, cuts: u32, smoothness: f32 },
    Bevel          { amount: f32, segments: u32, vertex: bool },
    Subdivide      { cuts: u32, smooth: f32 },
    Merge          { point: MergePoint },
    Bridge         { loops: u32, merge: bool },
    KnifeProject   { cut_through: bool },
    Fill           { triangulate: bool },
    Rip,
    SplitEdge,
    FlipNormals,
    RecalcNormals  { inside: bool },
    SetSmooth      { smooth: bool },
    MergeByDistance{ threshold: f32 },
    // Transform
    Translate      { delta: Vec3 },
    Rotate         { angle: f32, axis: Vec3 },
    Scale          { factor: Vec3 },
    // Selection
    SelectAll, SelectNone, SelectInverse,
    SelectLinked, SelectSimilar { similarity: String },
    // Sculpt
    SculptStroke   { brush: String, points: Vec<Vec3>, pressure: Vec<f32> },
    // UV
    UnwrapUV       { method: UnwrapMethod },
    // AI Operations
    AiGenerate     { prompt: String, result_path: String },
    AiRefine       { prompt: String, selection: Vec<u32> },
    AiRetopo       { source_mesh: String, polycount: u32 },
    AiTexture      { prompt: String, resolution: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergePoint { First, Last, Center, Cursor }

// ─── Display Settings ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub wireframe: bool,
    pub xray: bool,
    pub show_normals: bool,
    pub normal_length: f32,
    pub show_vertex_indices: bool,
    pub show_edge_lengths: bool,
    pub show_edge_angles: bool,
    pub overlay_opacity: f32,
    pub bone_axes: bool,
    pub viewport_shading: ViewportShading,
    pub matcap: Option<String>,
    pub cavity: bool,
    pub cavity_type: CavityType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewportShading { Solid, Material, Rendered, Wireframe }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CavityType { World, Screen, Both }

// ─── AI Modeler Agent ─────────────────────────────────────────────────────────

use async_trait::async_trait;
use anyhow::Result;
use chronoverse_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

pub struct AiModelingAgent {
    base: AgentBase,
    active_mesh: Option<String>,
    suggestion_queue: Vec<ModelingSuggestion>,
    reference_images: Vec<String>,
    current_style_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelingSuggestion {
    pub id: String,
    pub description: String,
    pub operation: ModelOpType,
    pub confidence: f32,
    pub reason: String,
}

impl AiModelingAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "ai_modeler".to_string(),
                name: "AI Modeling Agent".to_string(),
                description: "Guides 3D modeling process. Can generate models from text/images, auto-retopo, UV unwrap, and assist artists step-by-step.".to_string(),
                system_prompt: r#"You are the AI Modeling Agent inside ChronoVerse's built-in 3D modeler.

You help users create 3D models through:
1. TEXT TO 3D: Generate base meshes from descriptions
2. IMAGE TO 3D: Reconstruct 3D from reference photos
3. GUIDED MODELING: Suggest next steps as user models
4. AUTO-RETOPO: Clean mesh generation from sculpts
5. UV UNWRAPPING: Optimal seam placement
6. TEXTURE GENERATION: PBR textures from description
7. RIGGING ASSISTANCE: Auto-weight painting, bone placement

When helping a user:
- Suggest specific tool operations
- Explain WHY to use each technique
- Watch their mesh and warn about problems (non-manifold, ngons, etc.)
- Offer to auto-complete complex operations
- Generate reference images for guidance

For users with NO 3D skills:
1. Ask them to describe what they want
2. Generate a base mesh via GPU Lab
3. Help them refine with guided sculpting
4. Auto-retopo + UV unwrap
5. Generate textures from their description
6. Auto-rig if it's a character"#.to_string(),
                tools: vec![
                    "analyze_mesh".to_string(),
                    "suggest_operation".to_string(),
                    "generate_3d_base".to_string(),
                    "auto_retopo".to_string(),
                    "auto_uv_unwrap".to_string(),
                    "generate_texture".to_string(),
                    "auto_rig".to_string(),
                    "image_to_3d".to_string(),
                    "mesh_quality_check".to_string(),
                    "create_tool".to_string(),
                ],
                listens_to: vec!["mesh_changed".to_string(), "modeling_help_requested".to_string()],
                can_emit: vec!["modeling_suggestion".to_string(), "asset_3d_requested".to_string()],
                max_tokens_per_call: 2000,
                parallelizable: true,
                priority: 5,
            }),
            active_mesh: None,
            suggestion_queue: Vec::new(),
            reference_images: Vec::new(),
            current_style_prompt: String::new(),
        }
    }

    /// Analyze mesh quality and return issues
    pub fn check_mesh_quality(&self, mesh: &EditMesh) -> Vec<MeshIssue> {
        let mut issues = Vec::new();

        // Check for ngons (faces with >4 vertices — bad for subdivision/animation)
        for (i, face) in mesh.faces.iter().enumerate() {
            if face.vertices.len() > 4 {
                issues.push(MeshIssue {
                    severity: IssueSeverity::Warning,
                    description: format!("Face {} is an N-gon ({} verts). Convert to quads/tris for better subdivision.", i, face.vertices.len()),
                    affected_elements: vec![i as u32],
                    fix_available: true,
                    fix_description: "Triangulate N-gon".to_string(),
                });
            }
        }

        // Check vertex count
        if mesh.vertices.len() > 100_000 {
            issues.push(MeshIssue {
                severity: IssueSeverity::Warning,
                description: format!("High vertex count: {}. Consider decimation for real-time use.", mesh.vertices.len()),
                affected_elements: Vec::new(),
                fix_available: true,
                fix_description: "Run Decimate modifier (Ratio: 0.5)".to_string(),
            });
        }

        issues
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_elements: Vec<u32>,
    pub fix_available: bool,
    pub fix_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity { Info, Warning, Error }

#[async_trait]
impl Agent for AiModelingAgent {
    fn id(&self) -> &str { "ai_modeler" }
    fn name(&self) -> &str { "AI Modeling Agent" }
    fn description(&self) -> &str { "AI-powered 3D modeling assistance" }
    fn status(&self) -> &AgentStatus { &self.base.status }
    fn subscriptions(&self) -> Vec<&'static str> { vec!["mesh_changed", "analytics_event", "modeling_help_requested"] }
    fn tick_interval(&self) -> Option<f32> { Some(1.0) }

    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        if let AgentEvent::AnalyticsEvent { event_type, data } = &event.event {
            match event_type.as_str() {
                "modeling_help_requested" => {
                    let prompt = data.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
                    tracing::info!("AI Modeler: help requested: {}", prompt);
                    self.current_style_prompt = prompt.to_string();
                }
                "generate_3d_model" => {
                    let prompt = data.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
                    ctx.emit(AgentEvent::Asset3DRequested {
                        request_id: uuid::Uuid::new_v4().to_string(),
                        prompt: prompt.to_string(),
                        style: "realistic".to_string(),
                        auto_rig: false,
                    }, self.id());
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn tick(&mut self, ctx: &AgentContext, _: f32) -> Result<()> { Ok(()) }
}

// ─── Modeler State ────────────────────────────────────────────────────────────

pub struct ModelerState {
    pub meshes: HashMap<String, EditMesh>,
    pub selected_mesh: Option<String>,
    pub mode: EditMode,
    pub active_tool: String,
    pub undo_stack: Vec<ModelOperation>,
    pub redo_stack: Vec<ModelOperation>,
    pub undo_limit: usize,
    pub sculpt_brush: SculptBrush,
    pub weight_paint: WeightPaintState,
    pub symmetry: SymmetrySettings,
    pub snap_settings: SnapSettings,
    pub proportional_editing: ProportionalEditSettings,
    pub ai_agent: AiModelingAgent,
    pub session_start: std::time::Instant,
    pub auto_save_interval_secs: f32,
    pub last_save: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSettings {
    pub enabled: bool,
    pub snap_to: SnapElement,
    pub align_rotation: bool,
    pub snap_peel_object: bool,
    pub backface_culling: bool,
    pub face_nearest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapElement { Increment, Vertex, Edge, Face, Volume, EdgeCenter, EdgePerp, FaceCenter }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProportionalEditSettings {
    pub enabled: bool,
    pub connected_only: bool,
    pub projected_2d: bool,
    pub radius: f32,
    pub falloff: FalloffCurve,
}

impl ModelerState {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            selected_mesh: None,
            mode: EditMode::Object,
            active_tool: "move".to_string(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: 500,
            sculpt_brush: SculptBrush {
                name: "Draw".to_string(),
                brush_type: SculptBrushType::Draw,
                radius: 50.0, strength: 0.5, hardness: 0.7,
                auto_smooth: 0.0, normal_radius: 0.5,
                plane_offset: 0.0, plane_trim: 0.5,
                use_frontface: true,
                falloff: FalloffCurve::Smooth,
                stroke_method: StrokeMethod::Space { spacing: 0.1 },
                texture: None, texture_angle: 0.0, texture_rake: false,
            },
            weight_paint: WeightPaintState {
                active_group: None, brush_weight: 1.0,
                normalize_all: false, auto_normalize: true,
                multipaint: false, vertex_selection: false,
                zero_weights_action: "none".to_string(),
            },
            symmetry: SymmetrySettings { enabled: true, axis: SymmetryAxis::X, threshold: 0.001 },
            snap_settings: SnapSettings {
                enabled: false, snap_to: SnapElement::Vertex,
                align_rotation: true, snap_peel_object: false,
                backface_culling: false, face_nearest: false,
            },
            proportional_editing: ProportionalEditSettings {
                enabled: false, connected_only: false,
                projected_2d: false, radius: 1.0, falloff: FalloffCurve::Smooth,
            },
            ai_agent: AiModelingAgent::new(),
            session_start: std::time::Instant::now(),
            auto_save_interval_secs: 120.0,
            last_save: std::time::Instant::now(),
        }
    }

    pub fn add_mesh(&mut self, mesh: EditMesh) -> String {
        let id = mesh.id.clone();
        self.selected_mesh = Some(id.clone());
        self.meshes.insert(id.clone(), mesh);
        id
    }

    pub fn create_primitive(&mut self, primitive: ModelPrimitive) -> String {
        let mesh = primitive.to_edit_mesh();
        self.add_mesh(mesh)
    }

    pub fn push_undo(&mut self, op: ModelOperation) {
        if self.undo_stack.len() >= self.undo_limit {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(op);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<&ModelOperation> {
        let op = self.undo_stack.pop()?;
        self.redo_stack.push(op);
        self.redo_stack.last()
    }

    pub fn mesh_count(&self) -> usize { self.meshes.len() }
    pub fn selected_mesh(&self) -> Option<&EditMesh> {
        self.selected_mesh.as_ref().and_then(|id| self.meshes.get(id))
    }
}

/// Built-in primitives that can be created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelPrimitive {
    Cube { size: f32 },
    Sphere { radius: f32, segments: u32, rings: u32 },
    Cylinder { radius: f32, height: f32, vertices: u32 },
    Cone { radius1: f32, radius2: f32, depth: f32, vertices: u32 },
    Torus { major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32 },
    Plane { size: f32, subdivisions: u32 },
    Circle { radius: f32, vertices: u32, fill: bool },
    UVSphere { radius: f32, segments: u32, rings: u32 },
    IcoSphere { radius: f32, subdivisions: u32 },
    Capsule { radius: f32, height: f32, segments: u32 },
    Monkey,   // Blender's Suzanne mascot — also our mascot!
    Empty,
    Custom { mesh_data: Vec<u8> },
}

impl ModelPrimitive {
    pub fn to_edit_mesh(&self) -> EditMesh {
        EditMesh {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{:?}", self).split('{').next().unwrap_or("Mesh").trim().to_string(),
            vertices: Vec::new(), // In production: generate actual geometry
            edges: Vec::new(),
            faces: Vec::new(),
            uv_layers: vec![UvLayer { name: "UVMap".to_string(), active: true, uvs: Vec::new() }],
            vertex_colors: Vec::new(),
            material_slots: vec!["Material".to_string()],
            shape_keys: Vec::new(),
            modifiers: Vec::new(),
            symmetry: SymmetrySettings { enabled: false, axis: SymmetryAxis::X, threshold: 0.001 },
            display_settings: DisplaySettings {
                wireframe: false, xray: false, show_normals: false,
                normal_length: 0.1, show_vertex_indices: false,
                show_edge_lengths: false, show_edge_angles: false,
                overlay_opacity: 1.0, bone_axes: false,
                viewport_shading: ViewportShading::Solid,
                matcap: None, cavity: true, cavity_type: CavityType::Both,
            },
        }
    }
}

// Re-export base for sub-module use
use super::base;

extern crate uuid;
extern crate tracing;
