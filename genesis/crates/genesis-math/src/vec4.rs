//! Vec4
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec4 { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }
impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
    pub fn from_vec3(v: crate::vec3::Vec3, w: f32) -> Self { Self { x: v.x, y: v.y, z: v.z, w } }
    pub fn xyz(self) -> crate::vec3::Vec3 { crate::vec3::Vec3::new(self.x, self.y, self.z) }
    pub fn dot(self, o: Self) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z + self.w*o.w }
    pub fn to_array(self) -> [f32; 4] { [self.x, self.y, self.z, self.w] }
}
impl Default for Vec4 { fn default() -> Self { Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 } } }
impl From<[f32; 4]> for Vec4 { fn from(a: [f32; 4]) -> Self { Self::new(a[0], a[1], a[2], a[3]) } }
