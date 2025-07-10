use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Deserialize, Serialize,
)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
}
impl Vec2 {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, rhs: Self) -> usize {
        let (px, py) = (self.x as isize, self.y as isize);
        let (qx, qy) = (rhs.x as isize, rhs.y as isize);
        ((px - qx).pow(2) + (py - qy).pow(2)).isqrt() as usize
    }

    pub fn distance_f32(&self, rhs: Self) -> f32 {
        let (px, py) = (self.x as f32, self.y as f32);
        let (qx, qy) = (rhs.x as f32, rhs.y as f32);
        ((px - qx).powf(2.0) + (py - qy).powf(2.0)).sqrt()
    }

    pub fn lerp(&self, rhs: Self, t: f32) -> Self {
        let x = (t - 1.0) * self.x as f32 + t * rhs.x as f32;
        let y = (t - 1.0) * self.y as f32 + t * rhs.y as f32;
        Self {
            x: x as usize,
            y: y as usize,
        }
    }
}
impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
