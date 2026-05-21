//! Genesis Math Library
//!
//! Provides: Vec2, Vec3, Vec4, Mat4, Quaternion, AABB, Ray,
//! Noise functions (Perlin, Simplex), Easing functions,
//! Color types, Bezier curves.

pub mod vec2;
pub mod vec3;
pub mod vec4;
pub mod mat4;
pub mod quat;
pub mod aabb;
pub mod ray;
pub mod noise;
pub mod easing;
pub mod color;
pub mod bezier;
pub mod random;

pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
pub use mat4::Mat4;
pub use quat::Quaternion;
pub use aabb::Aabb;
pub use ray::Ray;
pub use color::Color;
pub use easing::*;
pub use noise::*;
pub use random::*;

/// Common mathematical constants
pub mod consts {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TWO_PI: f32 = PI * 2.0;
    pub const HALF_PI: f32 = PI * 0.5;
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;
    pub const EPSILON: f32 = 1e-6;
    pub const SQRT2: f32 = std::f32::consts::SQRT_2;
    pub const GRAVITY: f32 = 9.81;
}

/// Clamp a value between min and max
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Inverse linear interpolation — what t gives this value?
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < consts::EPSILON { 0.0 }
    else { (value - a) / (b - a) }
}

/// Remap value from [in_min, in_max] to [out_min, out_max]
pub fn remap(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    lerp(out_min, out_max, inverse_lerp(in_min, in_max, value))
}

/// Smooth step (Hermite interpolation)
pub fn smoothstep(a: f32, b: f32, t: f32) -> f32 {
    let t = clamp((t - a) / (b - a), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step (Ken Perlin's improved version)
pub fn smootherstep(a: f32, b: f32, t: f32) -> f32 {
    let t = clamp((t - a) / (b - a), 0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Convert degrees to radians
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * consts::DEG_TO_RAD
}

/// Convert radians to degrees
pub fn rad_to_deg(radians: f32) -> f32 {
    radians * consts::RAD_TO_DEG
}

/// Check if two floats are approximately equal
pub fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < consts::EPSILON
}
