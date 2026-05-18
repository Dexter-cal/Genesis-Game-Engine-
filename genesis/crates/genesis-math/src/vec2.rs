//! Vec2
use serde::{Serialize, Deserialize};
use std::ops::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn length(self) -> f32 { (self.x*self.x + self.y*self.y).sqrt() }
    pub fn normalize(self) -> Self {
        let l = self.length();
        if l < 1e-6 { Self::ZERO } else { Self::new(self.x/l, self.y/l) }
    }
    pub fn dot(self, o: Self) -> f32 { self.x*o.x + self.y*o.y }
    pub fn distance(self, o: Self) -> f32 { (self - o).length() }
    pub fn lerp(self, o: Self, t: f32) -> Self { self + (o - self) * t }
    pub fn angle(self) -> f32 { self.y.atan2(self.x) }
    pub fn from_angle(angle: f32) -> Self { Self::new(angle.cos(), angle.sin()) }
    pub fn perpendicular(self) -> Self { Self::new(-self.y, self.x) }
    pub fn to_array(self) -> [f32; 2] { [self.x, self.y] }
}

impl Add for Vec2 { type Output = Self; fn add(self, r: Self) -> Self { Self::new(self.x+r.x, self.y+r.y) } }
impl Sub for Vec2 { type Output = Self; fn sub(self, r: Self) -> Self { Self::new(self.x-r.x, self.y-r.y) } }
impl Mul<f32> for Vec2 { type Output = Self; fn mul(self, r: f32) -> Self { Self::new(self.x*r, self.y*r) } }
impl Div<f32> for Vec2 { type Output = Self; fn div(self, r: f32) -> Self { Self::new(self.x/r, self.y/r) } }
impl Neg for Vec2 { type Output = Self; fn neg(self) -> Self { Self::new(-self.x, -self.y) } }
impl Default for Vec2 { fn default() -> Self { Self::ZERO } }
