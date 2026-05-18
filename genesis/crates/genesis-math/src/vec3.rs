//! 3D Vector — the most used math type in the engine

use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO:    Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE:     Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP:      Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN:    Self = Self { x: 0.0, y:-1.0, z: 0.0 };
    pub const RIGHT:   Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const LEFT:    Self = Self { x:-1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z:-1.0 };
    pub const BACK:    Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    #[inline] pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    #[inline] pub fn splat(v: f32) -> Self { Self { x: v, y: v, z: v } }

    #[inline] pub fn from_array(a: [f32; 3]) -> Self { Self { x: a[0], y: a[1], z: a[2] } }
    #[inline] pub fn to_array(self) -> [f32; 3] { [self.x, self.y, self.z] }

    /// Length (magnitude)
    #[inline] pub fn length(self) -> f32 { self.length_squared().sqrt() }
    #[inline] pub fn length_squared(self) -> f32 { self.x*self.x + self.y*self.y + self.z*self.z }

    /// Normalize to unit length
    #[inline] pub fn normalize(self) -> Self {
        let len = self.length();
        if len < crate::consts::EPSILON { Self::ZERO } else { self / len }
    }

    /// Normalize, return None if zero vector
    pub fn try_normalize(self) -> Option<Self> {
        let len = self.length();
        if len < crate::consts::EPSILON { None } else { Some(self / len) }
    }

    /// Dot product
    #[inline] pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product
    #[inline] pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Distance to another point
    #[inline] pub fn distance(self, other: Self) -> f32 { (self - other).length() }
    #[inline] pub fn distance_squared(self, other: Self) -> f32 { (self - other).length_squared() }

    /// Linear interpolation
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Reflect around a normal
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    /// Component-wise min/max
    pub fn min(self, other: Self) -> Self {
        Self { x: self.x.min(other.x), y: self.y.min(other.y), z: self.z.min(other.z) }
    }
    pub fn max(self, other: Self) -> Self {
        Self { x: self.x.max(other.x), y: self.y.max(other.y), z: self.z.max(other.z) }
    }
    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    /// Absolute value per component
    pub fn abs(self) -> Self {
        Self { x: self.x.abs(), y: self.y.abs(), z: self.z.abs() }
    }

    /// Is this vector approximately zero?
    pub fn is_zero(self) -> bool { self.length_squared() < crate::consts::EPSILON }

    /// Project onto another vector
    pub fn project(self, onto: Self) -> Self {
        let d = onto.length_squared();
        if d < crate::consts::EPSILON { Self::ZERO }
        else { onto * (self.dot(onto) / d) }
    }

    /// Angle in radians between two vectors
    pub fn angle_between(self, other: Self) -> f32 {
        let cos_angle = self.normalize().dot(other.normalize()).clamp(-1.0, 1.0);
        cos_angle.acos()
    }

    /// Move toward target by max_delta
    pub fn move_toward(self, target: Self, max_delta: f32) -> Self {
        let dir = target - self;
        let dist = dir.length();
        if dist <= max_delta || dist < crate::consts::EPSILON {
            target
        } else {
            self + dir / dist * max_delta
        }
    }

    /// XZ distance (ignoring Y — useful for navmesh queries)
    pub fn distance_xz(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz).sqrt()
    }
}

// ─── Operator overloads ──────────────────────────────────────────────────────

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x+rhs.x, self.y+rhs.y, self.z+rhs.z) }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x-rhs.x, self.y-rhs.y, self.z-rhs.z) }
}
impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self { Self::new(self.x*rhs, self.y*rhs, self.z*rhs) }
}
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 { Vec3::new(rhs.x*self, rhs.y*self, rhs.z*self) }
}
impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self { Self::new(self.x/rhs, self.y/rhs, self.z/rhs) }
}
impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) { self.x += rhs.x; self.y += rhs.y; self.z += rhs.z; }
}
impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) { self.x -= rhs.x; self.y -= rhs.y; self.z -= rhs.z; }
}
impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) { self.x *= rhs; self.y *= rhs; self.z *= rhs; }
}

impl Default for Vec3 {
    fn default() -> Self { Self::ZERO }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(a: [f32; 3]) -> Self { Self::from_array(a) }
}
impl From<Vec3> for [f32; 3] {
    fn from(v: Vec3) -> [f32; 3] { v.to_array() }
}
impl From<(f32, f32, f32)> for Vec3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self { Self::new(x, y, z) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dot() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert!((a.dot(b)).abs() < 1e-6, "perpendicular vectors have 0 dot product");
    }
    #[test]
    fn test_cross() {
        let a = Vec3::RIGHT;
        let b = Vec3::FORWARD;
        let c = a.cross(b);
        // Right x Forward = Down in standard right-hand coord
        assert!((c.length() - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_normalize() {
        let v = Vec3::new(3.0, 0.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_lerp() {
        let a = Vec3::ZERO;
        let b = Vec3::ONE;
        let m = a.lerp(b, 0.5);
        assert!((m.x - 0.5).abs() < 1e-6);
    }
}
