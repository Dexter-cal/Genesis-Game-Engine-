//! ChronoVerse Renderer
//!
//! Built on WGPU — runs on Vulkan, Metal, DX12, and WebGPU.
//!
//! Features:
//! - Physically-Based Rendering (PBR) — albedo, normal, roughness, metallic, AO
//! - Deferred rendering pipeline
//! - Shadow mapping (cascaded for directional, cube maps for point lights)
//! - Screen-Space Ambient Occlusion (SSAO)
//! - Screen-Space Reflections (SSR)
//! - Bloom, depth-of-field, motion blur, chromatic aberration
//! - HDR rendering with tone mapping
//! - Multiple viewport support (split screen)
//! - Water simulation rendering (FFT-based ocean, rivers, rain interaction)
//! - Rain / snow particle renderer
//! - Volumetric fog and god rays
//! - Mesh rendering with LOD
//! - Instanced rendering for vegetation/crowds
//! - Skinned mesh animation
//! - GPU particle system
//! - Shader graph system
//! - Color palettes and cel shading
//! - Custom post-processing stack

pub mod context;
pub mod mesh;
pub mod material;
pub mod shader;
pub mod pipeline;
pub mod shadow;
pub mod pbr;
pub mod post_process;
pub mod particle;
pub mod water;
pub mod viewport;
pub mod sky;
pub mod debug_draw;
pub mod ui_renderer;
pub mod palette;
pub mod lod;
pub mod instancing;
pub mod skinning;
pub mod plugin;

pub use context::RenderContext;
pub use mesh::*;
pub use material::*;
pub use shader::*;
pub use viewport::*;
pub use palette::*;
pub use plugin::RenderPlugin;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use genesis_math::{vec3::Vec3, vec2::Vec2, color::Color, mat4::Mat4};

// ─── Mesh System ─────────────────────────────────────────────────────────────

/// Mesh primitive types — generated in code, no file needed
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveMesh {
    Cube,
    Sphere { subdivisions: u32 },
    Cylinder { segments: u32 },
    Cone { segments: u32 },
    Torus { major_segments: u32, minor_segments: u32, minor_radius: f32 },
    Plane { subdivisions_x: u32, subdivisions_z: u32 },
    Quad,
    IcoSphere { subdivisions: u32 },
    Capsule { segments: u32, rings: u32 },
    Arrow,
    Grid { size: u32 },
    // 2D shapes
    Circle2D { segments: u32 },
    Rect2D,
    Triangle2D,
    Ring2D { inner_radius: f32, segments: u32 },
}

impl PrimitiveMesh {
    /// Generate vertex data for this primitive
    pub fn generate(&self) -> MeshData {
        match self {
            Self::Cube => generate_cube(),
            Self::Quad => generate_quad(),
            Self::Plane { subdivisions_x, subdivisions_z } =>
                generate_plane(*subdivisions_x, *subdivisions_z),
            Self::Sphere { subdivisions } => generate_uv_sphere(*subdivisions),
            Self::Cylinder { segments } => generate_cylinder(*segments),
            Self::Capsule { segments, rings } => generate_capsule(*segments, *rings),
            _ => generate_cube(), // fallback
        }
    }
}

/// Raw mesh data (vertices, normals, UVs, indices)
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tangents: Vec<[f32; 4]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub bone_indices: Vec<[u32; 4]>,   // for skinning
    pub bone_weights: Vec<[f32; 4]>,   // for skinning
    pub primitive_type: MeshPrimitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MeshPrimitive {
    Triangles,
    Lines,
    Points,
    TriangleStrip,
    LineStrip,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            tangents: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            bone_indices: Vec::new(),
            bone_weights: Vec::new(),
            primitive_type: MeshPrimitive::Triangles,
        }
    }
}

impl MeshData {
    /// Calculate tangents from positions, UVs, and normals
    pub fn calculate_tangents(&mut self) {
        let n = self.vertices.len();
        self.tangents = vec![[0.0, 0.0, 1.0, 1.0]; n];

        if self.uvs.is_empty() || self.indices.len() < 3 { return; }

        let mut tangents = vec![[0.0f32; 3]; n];
        let mut bitangents = vec![[0.0f32; 3]; n];

        for tri in self.indices.chunks(3) {
            if tri.len() < 3 { continue; }
            let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);

            let v0 = Vec3::from(self.vertices[i0]);
            let v1 = Vec3::from(self.vertices[i1]);
            let v2 = Vec3::from(self.vertices[i2]);

            let uv0 = self.uvs[i0];
            let uv1 = self.uvs[i1];
            let uv2 = self.uvs[i2];

            let e1 = v1 - v0;
            let e2 = v2 - v0;
            let du1 = uv1[0] - uv0[0];
            let dv1 = uv1[1] - uv0[1];
            let du2 = uv2[0] - uv0[0];
            let dv2 = uv2[1] - uv0[1];

            let det = du1 * dv2 - du2 * dv1;
            if det.abs() < 1e-8 { continue; }
            let inv_det = 1.0 / det;

            let t = (e1 * dv2 - e2 * dv1) * inv_det;
            let b = (e2 * du1 - e1 * du2) * inv_det;

            for &i in &[i0, i1, i2] {
                let ot = &mut tangents[i];
                ot[0] += t.x; ot[1] += t.y; ot[2] += t.z;
                let ob = &mut bitangents[i];
                ob[0] += b.x; ob[1] += b.y; ob[2] += b.z;
            }
        }

        for i in 0..n {
            let n = Vec3::from(self.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]));
            let t = Vec3::from(tangents[i]);
            let b = Vec3::from(bitangents[i]);
            // Gram-Schmidt orthogonalize
            let t = (t - n * n.dot(t)).normalize();
            // Handedness
            let w = if n.cross(t).dot(b) < 0.0 { -1.0 } else { 1.0 };
            self.tangents[i] = [t.x, t.y, t.z, w];
        }
    }

    /// Compute AABB bounds
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for v in &self.vertices {
            let vv = Vec3::from(*v);
            min = min.min(vv);
            max = max.max(vv);
        }
        (min, max)
    }

    pub fn vertex_count(&self) -> usize { self.vertices.len() }
    pub fn triangle_count(&self) -> usize { self.indices.len() / 3 }
}

// ─── Primitive Generators ────────────────────────────────────────────────────

pub fn generate_cube() -> MeshData {
    let v = 0.5f32;
    let vertices = vec![
        // Front
        [-v,-v, v], [ v,-v, v], [ v, v, v], [-v, v, v],
        // Back
        [ v,-v,-v], [-v,-v,-v], [-v, v,-v], [ v, v,-v],
        // Left
        [-v,-v,-v], [-v,-v, v], [-v, v, v], [-v, v,-v],
        // Right
        [ v,-v, v], [ v,-v,-v], [ v, v,-v], [ v, v, v],
        // Top
        [-v, v, v], [ v, v, v], [ v, v,-v], [-v, v,-v],
        // Bottom
        [-v,-v,-v], [ v,-v,-v], [ v,-v, v], [-v,-v, v],
    ];
    let normals = vec![
        [0.,0.,1.],[0.,0.,1.],[0.,0.,1.],[0.,0.,1.],
        [0.,0.,-1.],[0.,0.,-1.],[0.,0.,-1.],[0.,0.,-1.],
        [-1.,0.,0.],[-1.,0.,0.],[-1.,0.,0.],[-1.,0.,0.],
        [1.,0.,0.],[1.,0.,0.],[1.,0.,0.],[1.,0.,0.],
        [0.,1.,0.],[0.,1.,0.],[0.,1.,0.],[0.,1.,0.],
        [0.,-1.,0.],[0.,-1.,0.],[0.,-1.,0.],[0.,-1.,0.],
    ];
    let uvs = vec![
        [0.,0.],[1.,0.],[1.,1.],[0.,1.],
        [0.,0.],[1.,0.],[1.,1.],[0.,1.],
        [0.,0.],[1.,0.],[1.,1.],[0.,1.],
        [0.,0.],[1.,0.],[1.,1.],[0.,1.],
        [0.,0.],[1.,0.],[1.,1.],[0.,1.],
        [0.,0.],[1.,0.],[1.,1.],[0.,1.],
    ];
    let mut indices = Vec::new();
    for face in 0..6u32 {
        let b = face * 4;
        indices.extend_from_slice(&[b,b+1,b+2, b,b+2,b+3]);
    }
    MeshData { vertices, normals, uvs, indices, ..Default::default() }
}

pub fn generate_quad() -> MeshData {
    MeshData {
        vertices: vec![[-0.5,-0.5,0.],[0.5,-0.5,0.],[0.5,0.5,0.],[-0.5,0.5,0.]],
        normals:  vec![[0.,0.,1.],[0.,0.,1.],[0.,0.,1.],[0.,0.,1.]],
        uvs:      vec![[0.,1.],[1.,1.],[1.,0.],[0.,0.]],
        indices:  vec![0,1,2,0,2,3],
        ..Default::default()
    }
}

pub fn generate_plane(sx: u32, sz: u32) -> MeshData {
    let (sx, sz) = (sx.max(1), sz.max(1));
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for z in 0..=sz {
        for x in 0..=sx {
            let fx = x as f32 / sx as f32 - 0.5;
            let fz = z as f32 / sz as f32 - 0.5;
            vertices.push([fx, 0.0, fz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([x as f32 / sx as f32, z as f32 / sz as f32]);
        }
    }

    for z in 0..sz {
        for x in 0..sx {
            let tl = z * (sx + 1) + x;
            let tr = tl + 1;
            let bl = tl + sx + 1;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    MeshData { vertices, normals, uvs, indices, ..Default::default() }
}

pub fn generate_uv_sphere(subdivisions: u32) -> MeshData {
    let rings = subdivisions.max(2);
    let sectors = subdivisions.max(3) * 2;
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    use std::f32::consts::PI;
    for r in 0..=rings {
        let phi = PI * r as f32 / rings as f32;
        for s in 0..=sectors {
            let theta = 2.0 * PI * s as f32 / sectors as f32;
            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();
            vertices.push([x * 0.5, y * 0.5, z * 0.5]);
            normals.push([x, y, z]);
            uvs.push([s as f32 / sectors as f32, r as f32 / rings as f32]);
        }
    }

    let stride = sectors + 1;
    for r in 0..rings {
        for s in 0..sectors {
            let tl = r * stride + s;
            let tr = tl + 1;
            let bl = tl + stride;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    MeshData { vertices, normals, uvs, indices, ..Default::default() }
}

pub fn generate_cylinder(segments: u32) -> MeshData {
    let segments = segments.max(3);
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    use std::f32::consts::PI;
    // Side
    for i in 0..=segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        let (c, s) = (angle.cos(), angle.sin());
        let u = i as f32 / segments as f32;
        vertices.push([c * 0.5, -0.5, s * 0.5]);
        normals.push([c, 0.0, s]);
        uvs.push([u, 0.0]);
        vertices.push([c * 0.5, 0.5, s * 0.5]);
        normals.push([c, 0.0, s]);
        uvs.push([u, 1.0]);
    }
    for i in 0..segments {
        let b = i * 2;
        indices.extend_from_slice(&[b, b+2, b+1, b+1, b+2, b+3]);
    }

    MeshData { vertices, normals, uvs, indices, ..Default::default() }
}

pub fn generate_capsule(segments: u32, rings: u32) -> MeshData {
    // Simplified — in production uses a proper capsule generator
    generate_uv_sphere(segments.max(4))
}
