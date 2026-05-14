//! Color type
use serde::{Serialize, Deserialize};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

impl Color {
    pub const WHITE:       Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK:       Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED:         Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN:       Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE:        Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW:      Self = Self { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const CYAN:        Self = Self { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA:     Self = Self { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const CHRONO_PURPLE: Self = Self { r: 0.424, g: 0.235, b: 0.882, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self { Self { r, g, b, a: 1.0 } }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8)  & 0xFF) as f32 / 255.0;
        let b = ((hex)       & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }

    pub fn from_hex_str(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        u32::from_str_radix(s, 16).ok().map(Self::from_hex)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    pub fn with_alpha(mut self, a: f32) -> Self { self.a = a; self }
    pub fn to_array(self) -> [f32; 4] { [self.r, self.g, self.b, self.a] }
    pub fn to_srgb_array(self) -> [f32; 4] {
        fn lin_to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1.0/2.4) - 0.055 }
        }
        [lin_to_srgb(self.r), lin_to_srgb(self.g), lin_to_srgb(self.b), self.a]
    }
}

impl Default for Color { fn default() -> Self { Self::WHITE } }
