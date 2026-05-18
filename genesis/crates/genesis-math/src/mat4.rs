//! 4x4 column-major matrix for 3D transforms
use serde::{Serialize, Deserialize};
use crate::vec3::Vec3;
use crate::vec4::Vec4;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Mat4 {
    /// Columns stored column-major (like OpenGL/WGPU)
    pub cols: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    };
    pub const ZERO: Self = Self { cols: [[0.0; 4]; 4] };

    pub fn translation(t: Vec3) -> Self {
        let mut m = Self::IDENTITY;
        m.cols[3][0] = t.x;
        m.cols[3][1] = t.y;
        m.cols[3][2] = t.z;
        m
    }

    pub fn scale(s: Vec3) -> Self {
        let mut m = Self::IDENTITY;
        m.cols[0][0] = s.x;
        m.cols[1][1] = s.y;
        m.cols[2][2] = s.z;
        m
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::IDENTITY;
        m.cols[1][1] = c;  m.cols[1][2] = s;
        m.cols[2][1] = -s; m.cols[2][2] = c;
        m
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::IDENTITY;
        m.cols[0][0] = c;  m.cols[0][2] = -s;
        m.cols[2][0] = s;  m.cols[2][2] = c;
        m
    }

    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        let mut m = Self::IDENTITY;
        m.cols[0][0] = c;  m.cols[0][1] = s;
        m.cols[1][0] = -s; m.cols[1][1] = c;
        m
    }

    pub fn perspective(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_radians * 0.5).tan();
        let range_inv = 1.0 / (near - far);
        let mut m = Self::ZERO;
        m.cols[0][0] = f / aspect;
        m.cols[1][1] = f;
        m.cols[2][2] = (near + far) * range_inv;
        m.cols[2][3] = -1.0;
        m.cols[3][2] = near * far * range_inv * 2.0;
        m
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);
        let mut m = Self::IDENTITY;
        m.cols[0][0] = r.x; m.cols[1][0] = r.y; m.cols[2][0] = r.z;
        m.cols[0][1] = u.x; m.cols[1][1] = u.y; m.cols[2][1] = u.z;
        m.cols[0][2] =-f.x; m.cols[1][2] =-f.y; m.cols[2][2] =-f.z;
        m.cols[3][0] = -r.dot(eye);
        m.cols[3][1] = -u.dot(eye);
        m.cols[3][2] = f.dot(eye);
        m
    }

    /// Matrix multiplication
    pub fn mul_mat4(self, rhs: Self) -> Self {
        let mut result = Self::ZERO;
        for col in 0..4 {
            for row in 0..4 {
                for k in 0..4 {
                    result.cols[col][row] += self.cols[k][row] * rhs.cols[col][k];
                }
            }
        }
        result
    }

    /// Transform a Vec3 (applies translation)
    pub fn transform_point(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.cols[0][0]*v.x + self.cols[1][0]*v.y + self.cols[2][0]*v.z + self.cols[3][0],
            self.cols[0][1]*v.x + self.cols[1][1]*v.y + self.cols[2][1]*v.z + self.cols[3][1],
            self.cols[0][2]*v.x + self.cols[1][2]*v.y + self.cols[2][2]*v.z + self.cols[3][2],
        )
    }

    /// Transform a direction vector (no translation)
    pub fn transform_vector(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.cols[0][0]*v.x + self.cols[1][0]*v.y + self.cols[2][0]*v.z,
            self.cols[0][1]*v.x + self.cols[1][1]*v.y + self.cols[2][1]*v.z,
            self.cols[0][2]*v.x + self.cols[1][2]*v.y + self.cols[2][2]*v.z,
        )
    }

    pub fn to_cols_array(self) -> [[f32; 4]; 4] { self.cols }
    pub fn as_slice(&self) -> &[f32; 16] { bytemuck::cast_ref(self) }
}

impl Default for Mat4 {
    fn default() -> Self { Self::IDENTITY }
}

impl std::ops::Mul for Mat4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { self.mul_mat4(rhs) }
}
