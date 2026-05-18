//! Ray for raycasting
use serde::{Serialize, Deserialize};
use crate::vec3::Vec3;
use crate::aabb::Aabb;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3, // normalized
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction: direction.normalize() }
    }
    pub fn at(&self, t: f32) -> Vec3 { self.origin + self.direction * t }

    /// Intersect with AABB — returns t value or None
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv = Vec3::new(1.0/self.direction.x, 1.0/self.direction.y, 1.0/self.direction.z);
        let t1 = (aabb.min - self.origin).to_array();
        let t2 = (aabb.max - self.origin).to_array();
        let [ix, iy, iz] = inv.to_array();
        let tmin = f32::max(f32::max(
            f32::min(t1[0]*ix, t2[0]*ix),
            f32::min(t1[1]*iy, t2[1]*iy)),
            f32::min(t1[2]*iz, t2[2]*iz));
        let tmax = f32::min(f32::min(
            f32::max(t1[0]*ix, t2[0]*ix),
            f32::max(t1[1]*iy, t2[1]*iy)),
            f32::max(t1[2]*iz, t2[2]*iz));
        if tmax >= tmin && tmax >= 0.0 { Some(tmin.max(0.0)) } else { None }
    }

    /// Intersect with infinite plane — returns t or None
    pub fn intersect_plane(&self, plane_normal: Vec3, plane_point: Vec3) -> Option<f32> {
        let denom = plane_normal.dot(self.direction);
        if denom.abs() < 1e-6 { return None; }
        let t = plane_normal.dot(plane_point - self.origin) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }

    /// Intersect with sphere
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b*b - 4.0*a*c;
        if discriminant < 0.0 { return None; }
        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t > 0.001 { Some(t) } else { None }
    }
}
