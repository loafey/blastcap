use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Deserialize, Serialize,
)]
pub struct Vec3 {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
impl Vec3 {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn distance(&self, rhs: Self) -> usize {
        let (px, py, pz) = (self.x as isize, self.y as isize, self.z as isize);
        let (qx, qy, qz) = (rhs.x as isize, rhs.y as isize, rhs.z as isize);
        ((px - qx).pow(2) + (py - qy).pow(2) + (pz - qz).pow(2)).isqrt() as usize
    }

    pub fn distance_f32(&self, rhs: Self) -> f32 {
        let (px, py, pz) = (self.x as f32, self.y as f32, self.z as f32);
        let (qx, qy, qz) = (rhs.x as f32, rhs.y as f32, rhs.z as f32);
        ((px - qx).powf(2.0) + (py - qy).powf(2.0) + (pz - qz).powf(2.0)).sqrt()
    }
}
impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
