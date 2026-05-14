//! Quaternion for smooth rotations
use serde::{Serialize, Deserialize};
use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Quaternion { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Quaternion {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let (s, c) = (angle * 0.5).sin_cos();
        let a = axis.normalize();
        Self::new(a.x * s, a.y * s, a.z * s, c)
    }

    pub fn from_euler_xyz(x: f32, y: f32, z: f32) -> Self {
        let (sx, cx) = (x * 0.5).sin_cos();
        let (sy, cy) = (y * 0.5).sin_cos();
        let (sz, cz) = (z * 0.5).sin_cos();
        Self::new(
            sx*cy*cz + cx*sy*sz,
            cx*sy*cz - sx*cy*sz,
            cx*cy*sz + sx*sy*cz,
            cx*cy*cz - sx*sy*sz,
        )
    }

    pub fn length(self) -> f32 { (self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w).sqrt() }

    pub fn normalize(self) -> Self {
        let l = self.length();
        if l < 1e-6 { Self::IDENTITY }
        else { Self::new(self.x/l, self.y/l, self.z/l, self.w/l) }
    }

    pub fn conjugate(self) -> Self { Self::new(-self.x, -self.y, -self.z, self.w) }

    pub fn mul_quat(self, rhs: Self) -> Self {
        Self::new(
            self.w*rhs.x + self.x*rhs.w + self.y*rhs.z - self.z*rhs.y,
            self.w*rhs.y - self.x*rhs.z + self.y*rhs.w + self.z*rhs.x,
            self.w*rhs.z + self.x*rhs.y - self.y*rhs.x + self.z*rhs.w,
            self.w*rhs.w - self.x*rhs.x - self.y*rhs.y - self.z*rhs.z,
        ).normalize()
    }

    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let uv = qv.cross(v);
        let uuv = qv.cross(uv);
        v + (uv * 2.0 * self.w) + uuv * 2.0
    }

    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.x*other.x + self.y*other.y + self.z*other.z + self.w*other.w;
        let other = if dot < 0.0 { dot = -dot; Self::new(-other.x, -other.y, -other.z, -other.w) } else { other };
        if dot > 0.9995 {
            let r = Self::new(
                self.x + t*(other.x - self.x), self.y + t*(other.y - self.y),
                self.z + t*(other.z - self.z), self.w + t*(other.w - self.w),
            );
            return r.normalize();
        }
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let (st, ct) = theta.sin_cos();
        let s0 = ct - dot * st / theta_0.sin();
        let s1 = st / theta_0.sin();
        Self::new(
            s0*self.x + s1*other.x, s0*self.y + s1*other.y,
            s0*self.z + s1*other.z, s0*self.w + s1*other.w,
        )
    }

    pub fn forward(self) -> Vec3 { self.rotate_vec3(Vec3::FORWARD) }
    pub fn right(self)   -> Vec3 { self.rotate_vec3(Vec3::RIGHT) }
    pub fn up(self)      -> Vec3 { self.rotate_vec3(Vec3::UP) }
    pub fn to_array(self) -> [f32; 4] { [self.x, self.y, self.z, self.w] }
}

impl Default for Quaternion { fn default() -> Self { Self::IDENTITY } }
impl std::ops::Mul for Quaternion {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { self.mul_quat(rhs) }
}
