//! AABB - Axis-Aligned Bounding Box
use serde::{Serialize, Deserialize};
use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self { Self { min, max } }
    pub fn from_center_half_extents(center: Vec3, half: Vec3) -> Self {
        Self { min: center - half, max: center + half }
    }
    pub fn center(&self) -> Vec3 { (self.min + self.max) * 0.5 }
    pub fn half_extents(&self) -> Vec3 { (self.max - self.min) * 0.5 }
    pub fn size(&self) -> Vec3 { self.max - self.min }
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
    pub fn expand(&self, amount: f32) -> Self {
        let v = Vec3::splat(amount);
        Self { min: self.min - v, max: self.max + v }
    }
    pub fn merge(&self, other: &Aabb) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }
    pub fn surface_area(&self) -> f32 {
        let s = self.size();
        2.0 * (s.x*s.y + s.y*s.z + s.z*s.x)
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}
