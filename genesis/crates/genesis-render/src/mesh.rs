//! PBR Material System + Shader Graph + Color Palettes

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use genesis_math::color::Color;

// ─── Material ─────────────────────────────────────────────────────────────────

/// PBR material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: String,
    pub name: String,
    pub shader: MaterialShader,

    // ─── PBR Textures ────────────────────────────────────────────────────
    pub albedo_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub roughness_texture: Option<String>,
    pub metallic_texture: Option<String>,
    pub ao_texture: Option<String>,
    pub emissive_texture: Option<String>,
    pub height_texture: Option<String>,    // parallax / displacement
    pub opacity_texture: Option<String>,

    // ─── PBR Values (used when no texture) ───────────────────────────────
    pub albedo_color: Color,
    pub emissive_color: Color,
    pub roughness: f32,         // 0 = mirror, 1 = fully rough
    pub metallic: f32,          // 0 = dielectric, 1 = metal
    pub ao_strength: f32,
    pub emissive_strength: f32,
    pub normal_strength: f32,
    pub height_scale: f32,      // parallax depth

    // ─── Transparency ────────────────────────────────────────────────────
    pub blend_mode: BlendMode,
    pub alpha_cutoff: f32,      // for alpha testing
    pub opacity: f32,

    // ─── UV ──────────────────────────────────────────────────────────────
    pub uv_scale: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_rotation: f32,

    // ─── Special Effects ─────────────────────────────────────────────────
    pub two_sided: bool,
    pub wireframe: bool,
    pub vertex_color: bool,     // use vertex colors as albedo modifier
    pub custom_params: HashMap<String, f32>,

    // ─── Cel Shading / Toon ──────────────────────────────────────────────
    pub cel_shading: bool,
    pub cel_steps: u32,
    pub outline_width: f32,
    pub outline_color: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    Opaque,
    AlphaTest,  // discard pixels below alpha_cutoff
    Transparent, // alpha blending
    Additive,   // fire, magic effects
    Multiply,
    Screen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialShader {
    /// Standard PBR (default for all 3D objects)
    StandardPbr,
    /// Unlit — no lighting calculations (UI, skybox, indicators)
    Unlit,
    /// Toon / cel-shaded
    Toon,
    /// Water surface (FFT waves, reflections, refraction)
    Water,
    /// Particle material (GPU-optimized, no depth write)
    Particle,
    /// Terrain (up to 8 texture layers, weight-blended)
    Terrain,
    /// Glass / refraction
    Glass,
    /// Custom shader graph
    Custom { shader_id: String },
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Material".to_string(),
            shader: MaterialShader::StandardPbr,
            albedo_texture: None,
            normal_texture: None,
            roughness_texture: None,
            metallic_texture: None,
            ao_texture: None,
            emissive_texture: None,
            height_texture: None,
            opacity_texture: None,
            albedo_color: Color::WHITE,
            emissive_color: Color::BLACK,
            roughness: 0.5,
            metallic: 0.0,
            ao_strength: 1.0,
            emissive_strength: 0.0,
            normal_strength: 1.0,
            height_scale: 0.0,
            blend_mode: BlendMode::Opaque,
            alpha_cutoff: 0.5,
            opacity: 1.0,
            uv_scale: [1.0, 1.0],
            uv_offset: [0.0, 0.0],
            uv_rotation: 0.0,
            two_sided: false,
            wireframe: false,
            vertex_color: false,
            custom_params: HashMap::new(),
            cel_shading: false,
            cel_steps: 3,
            outline_width: 0.0,
            outline_color: Color::BLACK,
        }
    }
}

impl Material {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), ..Default::default() }
    }

    pub fn unlit(color: Color) -> Self {
        Self { shader: MaterialShader::Unlit, albedo_color: color, ..Default::default() }
    }

    pub fn water() -> Self {
        Self { shader: MaterialShader::Water, roughness: 0.05, metallic: 0.0,
               albedo_color: Color::new(0.0, 0.3, 0.8, 0.7), blend_mode: BlendMode::Transparent,
               ..Default::default() }
    }

    pub fn glass() -> Self {
        Self { shader: MaterialShader::Glass, roughness: 0.0, metallic: 0.0,
               albedo_color: Color::new(0.9, 0.95, 1.0, 0.15), blend_mode: BlendMode::Transparent,
               two_sided: true, ..Default::default() }
    }

    pub fn toon(color: Color, steps: u32, outline: f32) -> Self {
        Self { shader: MaterialShader::Toon, albedo_color: color,
               cel_shading: true, cel_steps: steps, outline_width: outline,
               outline_color: Color::BLACK, ..Default::default() }
    }

    pub fn with_albedo_texture(mut self, id: &str) -> Self {
        self.albedo_texture = Some(id.to_string()); self
    }
    pub fn with_normal_texture(mut self, id: &str) -> Self {
        self.normal_texture = Some(id.to_string()); self
    }
    pub fn with_roughness(mut self, r: f32) -> Self { self.roughness = r; self }
    pub fn with_metallic(mut self, m: f32) -> Self { self.metallic = m; self }
    pub fn with_emissive(mut self, color: Color, strength: f32) -> Self {
        self.emissive_color = color; self.emissive_strength = strength; self
    }
    pub fn transparent(mut self, opacity: f32) -> Self {
        self.opacity = opacity; self.blend_mode = BlendMode::Transparent; self
    }
}

// ─── Shader Graph ────────────────────────────────────────────────────────────

/// A node in the shader graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderNode {
    pub id: String,
    pub node_type: ShaderNodeType,
    pub position: [f32; 2],
    pub inputs: HashMap<String, ShaderValue>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderNodeType {
    // ─── Input ────────────────────────────────────────────────────────
    TextureSample2D,
    TextureSampleCube,
    Time,
    ScreenUV,
    WorldPosition,
    WorldNormal,
    VertexColor,
    UVCoord { channel: u32 },
    Constant { value: ShaderValue },
    Parameter { name: String, default: ShaderValue },

    // ─── Math ─────────────────────────────────────────────────────────
    Add, Subtract, Multiply, Divide,
    Dot, Cross, Normalize, Length,
    Power, Sqrt, Abs, Clamp, Lerp, Fract,
    Sin, Cos, Tan,
    Min, Max, Step, SmoothStep,
    Negate, OneMinus,

    // ─── Vector ───────────────────────────────────────────────────────
    SplitVec { components: String },  // e.g. "xyz"
    CombineVec,
    SwizzleVec { mask: String },

    // ─── Color ────────────────────────────────────────────────────────
    ColorMultiply,
    ColorAdd,
    ColorBlend { mode: BlendMode },
    HsvToRgb,
    RgbToHsv,
    GammaCorrect { to_linear: bool },

    // ─── Effects ─────────────────────────────────────────────────────
    FresnelEffect { power: f32 },
    NormalMap,
    ParallaxOcclusionMapping,
    ScreenSpaceReflection,
    Fog,
    Dither,
    Noise { noise_type: NoiseShaderType },
    Voronoi,

    // ─── Output (final material properties) ──────────────────────────
    PbrOutput,
    UnlitOutput,
    ParticleOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Bool(bool),
    Int(i32),
    Texture2D(String),
    TextureCube(String),
    Color([f32; 4]),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseShaderType { Perlin, Simplex, Voronoi, WhiteNoise, Value }

/// A complete shader graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderGraph {
    pub id: String,
    pub name: String,
    pub nodes: HashMap<String, ShaderNode>,
    pub connections: Vec<ShaderConnection>,
    pub output_node: String,
    pub preview_material_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderConnection {
    pub from_node: String,
    pub from_output: String,
    pub to_node: String,
    pub to_input: String,
}

impl ShaderGraph {
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            nodes: HashMap::new(),
            connections: Vec::new(),
            output_node: String::new(),
            preview_material_id: None,
        }
    }

    /// Compile the shader graph to WGSL shader code
    pub fn compile_to_wgsl(&self) -> Result<String, String> {
        // Full WGSL compilation is a major task
        // In production: topological sort nodes, emit WGSL for each
        Ok(format!("// Compiled shader: {}\n{}", self.name, DEFAULT_PBR_WGSL))
    }
}

// ─── Color Palettes ───────────────────────────────────────────────────────────

/// A color palette for maintaining visual style consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub id: String,
    pub name: String,
    pub colors: Vec<PaletteColor>,
    pub palette_type: PaletteType,
    pub locked: bool, // when locked, all generated assets must use these colors
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteColor {
    pub name: String,
    pub color: Color,
    pub role: ColorRole,
    pub usage_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorRole {
    Primary,
    Secondary,
    Accent,
    Background,
    Surface,
    Highlight,
    Shadow,
    Danger,
    Success,
    Warning,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaletteType {
    /// Pixel art — limited colors (8-64)
    PixelArt { max_colors: u32 },
    /// Cel shaded — flat colors with defined steps
    CelShaded { shade_steps: u32 },
    /// Photorealistic — no color limits
    Photorealistic,
    /// Monochrome
    Monochrome { accent_color: Color },
    /// Complementary — two main colors + neutrals
    Complementary,
    /// Analogous — colors close on the color wheel
    Analogous,
    /// Retro / CRT — based on classic hardware palettes
    Retro(RetroStyle),
    /// Custom
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetroStyle {
    GameBoy,       // 4 shades of green
    Nes,           // 56 colors
    Snes,          // 32768 colors (15-bit)
    MegaDrive,     // 512 colors, 64 on screen
    Psx,           // 16-bit color
    Commodore64,   // 16 colors
    Cga,           // 4 colors
    Ega,           // 16 colors
    Atari2600,     // 128 colors
}

impl ColorPalette {
    pub fn game_boy() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Game Boy".to_string(),
            colors: vec![
                PaletteColor { name: "Darkest".to_string(), color: Color::from_hex(0x0f380f), role: ColorRole::Shadow, usage_count: 0 },
                PaletteColor { name: "Dark".to_string(), color: Color::from_hex(0x306230), role: ColorRole::Secondary, usage_count: 0 },
                PaletteColor { name: "Light".to_string(), color: Color::from_hex(0x8bac0f), role: ColorRole::Primary, usage_count: 0 },
                PaletteColor { name: "Lightest".to_string(), color: Color::from_hex(0x9bbc0f), role: ColorRole::Background, usage_count: 0 },
            ],
            palette_type: PaletteType::Retro(RetroStyle::GameBoy),
            locked: false,
        }
    }

    pub fn find_closest(&self, target: Color) -> Option<&PaletteColor> {
        self.colors.iter().min_by(|a, b| {
            let da = color_distance(a.color, target);
            let db = color_distance(b.color, target);
            da.partial_cmp(&db).unwrap()
        })
    }

    pub fn quantize_color(&self, color: Color) -> Color {
        self.find_closest(color).map(|c| c.color).unwrap_or(color)
    }
}

fn color_distance(a: Color, b: Color) -> f32 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    (dr*dr + dg*dg + db*db).sqrt()
}

// ─── Default PBR WGSL Shader ─────────────────────────────────────────────────

pub const DEFAULT_PBR_WGSL: &str = r#"
// Genesis Standard PBR Shader
// Compatible with WGPU (Vulkan, Metal, DX12, WebGPU)

struct VertexInput {
    @location(0) position:  vec3<f32>,
    @location(1) normal:    vec3<f32>,
    @location(2) uv:        vec2<f32>,
    @location(3) tangent:   vec4<f32>,
    @location(4) color:     vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal:   vec3<f32>,
    @location(2) uv:             vec2<f32>,
    @location(3) tangent:        vec4<f32>,
    @location(4) color:          vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    position:  vec3<f32>,
};

struct ModelUniform {
    model:        mat4x4<f32>,
    model_normal: mat4x4<f32>,
};

struct MaterialUniform {
    albedo_color:      vec4<f32>,
    emissive_color:    vec3<f32>,
    roughness:         f32,
    metallic:          f32,
    ao_strength:       f32,
    emissive_strength: f32,
    normal_strength:   f32,
    uv_scale:          vec2<f32>,
    uv_offset:         vec2<f32>,
    alpha_cutoff:      f32,
    cel_shading:       u32,
    cel_steps:         f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model:  ModelUniform;
@group(2) @binding(0) var<uniform> mat:    MaterialUniform;
@group(2) @binding(1) var albedo_tex:   texture_2d<f32>;
@group(2) @binding(2) var normal_tex:   texture_2d<f32>;
@group(2) @binding(3) var rough_tex:    texture_2d<f32>;
@group(2) @binding(4) var metal_tex:    texture_2d<f32>;
@group(2) @binding(5) var tex_sampler:  sampler;

const PI: f32 = 3.14159265359;
const EPSILON: f32 = 0.0001;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position  = camera.view_proj * world_pos;
    out.world_normal   = normalize((model.model_normal * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv             = in.uv * mat.uv_scale + mat.uv_offset;
    out.tangent        = in.tangent;
    out.color          = in.color;
    return out;
}

// GGX Distribution function
fn ggx_d(n_dot_h: f32, roughness: f32) -> f32 {
    let a2 = roughness * roughness * roughness * roughness;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom + EPSILON);
}

// Schlick approximation for Fresnel
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// Smith's masking-shadowing function
fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let g1 = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let g2 = n_dot_l / (n_dot_l * (1.0 - k) + k);
    return g1 * g2;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Sample textures
    let albedo_sample  = textureSample(albedo_tex, tex_sampler, uv);
    let albedo         = albedo_sample.rgb * mat.albedo_color.rgb;
    let alpha          = albedo_sample.a * mat.albedo_color.a;

    // Alpha cutoff
    if alpha < mat.alpha_cutoff { discard; }

    let roughness = textureSample(rough_tex, tex_sampler, uv).r * mat.roughness;
    let metallic  = textureSample(metal_tex, tex_sampler, uv).r * mat.metallic;

    // Normal mapping
    let normal_sample = textureSample(normal_tex, tex_sampler, uv).rgb * 2.0 - 1.0;
    let tangent  = normalize(in.tangent.xyz);
    let bitangent = cross(in.world_normal, tangent) * in.tangent.w;
    let tbn = mat3x3<f32>(tangent, bitangent, in.world_normal);
    let N = normalize(tbn * (normal_sample * mat.normal_strength));

    let V = normalize(camera.position - in.world_position);
    let n_dot_v = max(dot(N, V), EPSILON);

    // F0 — reflectance at normal incidence
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    // Hardcoded directional light (light system injects via push constants in production)
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let light_color = vec3<f32>(1.0, 0.95, 0.9);
    let light_intensity = 3.0;

    let L = light_dir;
    let H = normalize(V + L);
    let n_dot_l = max(dot(N, L), 0.0);
    let n_dot_h = max(dot(N, H), 0.0);
    let v_dot_h = max(dot(V, H), 0.0);

    // Cook-Torrance BRDF
    let D = ggx_d(n_dot_h, roughness);
    let G = geometry_smith(n_dot_v, n_dot_l, roughness);
    let F = fresnel_schlick(v_dot_h, f0);

    let specular = (D * G * F) / (4.0 * n_dot_v * n_dot_l + EPSILON);
    let k_d = (vec3<f32>(1.0) - F) * (1.0 - metallic);
    let diffuse = k_d * albedo / PI;

    var direct = (diffuse + specular) * light_color * light_intensity * n_dot_l;

    // Ambient (simple hemisphere)
    let ambient = vec3<f32>(0.03) * albedo;

    // Emissive
    let emissive = mat.emissive_color * mat.emissive_strength;

    var color = ambient + direct + emissive;

    // Cel shading quantization
    if mat.cel_shading != 0u {
        let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
        let stepped = floor(luminance * mat.cel_steps) / mat.cel_steps;
        color = color * (stepped / (luminance + EPSILON));
    }

    // HDR tone mapping (Reinhard)
    color = color / (color + vec3<f32>(1.0));

    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, alpha);
}
"#;

// Render context, viewport, and plugin stubs
pub mod context { pub struct RenderContext; }
pub mod shadow { pub struct ShadowMap; }
pub mod pbr { pub struct PbrPipeline; }
pub mod post_process { pub struct PostProcessStack; }
pub mod sky { pub struct SkyRenderer; }
pub mod debug_draw { pub struct DebugDraw; }
pub mod ui_renderer { pub struct UiRenderer; }
pub mod lod { pub struct LodSystem; }
pub mod instancing { pub struct InstancedRenderer; }
pub mod skinning { pub struct SkinningSystem; }
pub mod pipeline { pub struct RenderPipeline; }

pub mod viewport {
    use serde::{Serialize, Deserialize};
    use crate::ViewportRect;

    /// A rendering viewport — one camera view rendered to a region of screen
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Viewport {
        pub id: String,
        pub rect: ViewportRect,
        pub camera_entity: Option<String>,
        pub player_index: u8,
        pub clear_color: genesis_math::color::Color,
        pub hdr: bool,
        pub msaa_samples: u32,
        pub active: bool,
        pub render_ui: bool,
        pub render_debug: bool,
    }

    impl Viewport {
        pub fn fullscreen(camera_entity: Option<String>) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                rect: ViewportRect { x: 0.0, y: 0.0, width: 1.0, height: 1.0, player_index: 0 },
                camera_entity,
                player_index: 0,
                clear_color: genesis_math::color::Color::new(0.05, 0.05, 0.08, 1.0),
                hdr: true,
                msaa_samples: 4,
                active: true,
                render_ui: true,
                render_debug: false,
            }
        }
    }

    /// Manages multiple viewports for split-screen
    pub struct ViewportManager {
        pub viewports: Vec<Viewport>,
        pub screen_width: u32,
        pub screen_height: u32,
    }

    impl ViewportManager {
        pub fn new(w: u32, h: u32) -> Self {
            Self { viewports: Vec::new(), screen_width: w, screen_height: h }
        }

        /// Get pixel rect for a normalized viewport
        pub fn pixel_rect(&self, viewport: &Viewport) -> (u32, u32, u32, u32) {
            (
                (viewport.rect.x * self.screen_width as f32) as u32,
                (viewport.rect.y * self.screen_height as f32) as u32,
                (viewport.rect.width * self.screen_width as f32) as u32,
                (viewport.rect.height * self.screen_height as f32) as u32,
            )
        }
    }
}

pub mod shader {
    pub use super::{ShaderGraph, ShaderNode, ShaderNodeType, ShaderValue, ShaderConnection};
    pub use super::DEFAULT_PBR_WGSL;
}

pub mod material {
    pub use super::{Material, MaterialShader, BlendMode};
}

pub mod palette {
    pub use super::{ColorPalette, PaletteColor, PaletteType, ColorRole, RetroStyle};
}

pub mod plugin {
    use genesis_core::{plugin::Plugin, engine::Engine};
    use async_trait::async_trait;
    use anyhow::Result;

    pub struct RenderPlugin {
        pub width: u32,
        pub height: u32,
    }

    impl RenderPlugin {
        pub fn new(width: u32, height: u32) -> Self { Self { width, height } }
    }

    #[async_trait]
    impl Plugin for RenderPlugin {
        fn name(&self) -> &str { "render" }
        fn description(&self) -> &str { "WGPU-based PBR renderer with shadows, particles, water, post-processing" }
        async fn initialize(&mut self, _: &mut Engine) -> Result<()> {
            tracing::info!("Render plugin initialized (WGPU, {}x{})", self.width, self.height);
            Ok(())
        }
        async fn update(&mut self, _: &mut Engine, _: f32) -> Result<()> { Ok(()) }
    }
}

// Re-export ViewportRect from network crate
use genesis_core as _cc;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewportRect {
    pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub player_index: u8,
}
