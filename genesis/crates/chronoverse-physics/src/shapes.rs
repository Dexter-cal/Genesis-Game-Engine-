//! Collision shapes — all shape types ChronoVerse supports
use rapier3d::prelude::*;
use chronoverse_math::vec3::Vec3;
use serde::{Serialize, Deserialize};

/// All collision shape configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollisionShape {
    /// Simple box (cheapest, preferred for props)
    Box    { half_x: f32, half_y: f32, half_z: f32 },
    /// Sphere (cheapest rotation-invariant shape)
    Sphere { radius: f32 },
    /// Capsule (best for characters)
    Capsule { half_height: f32, radius: f32 },
    /// Cylinder
    Cylinder { half_height: f32, radius: f32 },
    /// Cone
    Cone { half_height: f32, radius: f32 },
    /// Triangle mesh (most expensive, static only, exact shape)
    TriangleMesh { vertices: Vec<[f32; 3]>, indices: Vec<[u32; 3]> },
    /// Convex hull (generated from mesh vertices)
    ConvexHull { points: Vec<[f32; 3]> },
    /// Heightfield (for terrain)
    HeightField { heights: Vec<f32>, rows: u32, cols: u32, scale: [f32; 3] },
    /// Multiple shapes combined
    Compound { children: Vec<(Transform3D, CollisionShape)> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform3D {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

impl CollisionShape {
    /// Convert to Rapier SharedShape
    pub fn to_rapier(&self) -> SharedShape {
        match self {
            Self::Box { half_x, half_y, half_z } =>
                SharedShape::cuboid(*half_x, *half_y, *half_z),

            Self::Sphere { radius } =>
                SharedShape::ball(*radius),

            Self::Capsule { half_height, radius } =>
                SharedShape::capsule_y(*half_height, *radius),

            Self::Cylinder { half_height, radius } =>
                SharedShape::cylinder(*half_height, *radius),

            Self::Cone { half_height, radius } =>
                SharedShape::cone(*half_height, *radius),

            Self::TriangleMesh { vertices, indices } => {
                let verts: Vec<rapier3d::math::Point<f32>> = vertices.iter()
                    .map(|v| rapier3d::math::point![v[0], v[1], v[2]])
                    .collect();
                let tris: Vec<[u32; 3]> = indices.clone();
                SharedShape::trimesh(verts, tris).unwrap_or(SharedShape::ball(1.0))
            }

            Self::ConvexHull { points } => {
                let pts: Vec<rapier3d::math::Point<f32>> = points.iter()
                    .map(|p| rapier3d::math::point![p[0], p[1], p[2]])
                    .collect();
                SharedShape::convex_hull(&pts).unwrap_or(SharedShape::ball(1.0))
            }

            Self::HeightField { heights, rows, cols, scale } => {
                let matrix = rapier3d::na::DMatrix::from_row_slice(
                    *rows as usize, *cols as usize, heights
                );
                SharedShape::heightfield(
                    matrix,
                    rapier3d::math::vector![scale[0], scale[1], scale[2]],
                )
            }

            Self::Compound { children } => {
                let shapes: Vec<(rapier3d::math::Isometry<f32>, SharedShape)> = children.iter()
                    .map(|(t, s)| {
                        let iso = rapier3d::math::Isometry::from_parts(
                            rapier3d::math::Translation::new(t.position[0], t.position[1], t.position[2]),
                            rapier3d::na::UnitQuaternion::new_normalize(
                                rapier3d::na::Quaternion::new(t.rotation[3], t.rotation[0], t.rotation[1], t.rotation[2])
                            ),
                        );
                        (iso, s.to_rapier())
                    })
                    .collect();
                SharedShape::compound(shapes)
            }
        }
    }

    /// Estimate mass from shape and density
    pub fn estimate_mass(&self, density: f32) -> f32 {
        let volume = self.approximate_volume();
        density * volume
    }

    pub fn approximate_volume(&self) -> f32 {
        use std::f32::consts::PI;
        match self {
            Self::Box { half_x, half_y, half_z } => 8.0 * half_x * half_y * half_z,
            Self::Sphere { radius } => (4.0/3.0) * PI * radius * radius * radius,
            Self::Capsule { half_height, radius } => {
                PI * radius * radius * (2.0 * half_height + (4.0/3.0) * radius)
            }
            Self::Cylinder { half_height, radius } => 2.0 * PI * radius * radius * half_height,
            Self::Cone { half_height, radius } => PI * radius * radius * half_height / 3.0,
            _ => 1.0,
        }
    }
}

/// Default shapes for common object types
impl CollisionShape {
    pub fn character_capsule() -> Self {
        Self::Capsule { half_height: 0.9, radius: 0.35 }
    }
    pub fn unit_box() -> Self {
        Self::Box { half_x: 0.5, half_y: 0.5, half_z: 0.5 }
    }
    pub fn unit_sphere() -> Self {
        Self::Sphere { radius: 0.5 }
    }
    pub fn crate_box(size: f32) -> Self {
        let h = size * 0.5;
        Self::Box { half_x: h, half_y: h, half_z: h }
    }
}
