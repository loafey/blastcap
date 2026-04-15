use std::{
    fmt::Display,
    ops::{Add, Div},
};

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Deserialize, Serialize,
)]
pub struct Vec3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}
impl Vec3 {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    pub fn distance(&self, rhs: Self) -> f64 {
        let (px, py, pz) = (self.x as isize, self.y as isize, self.z as isize);
        let (qx, qy, qz) = (rhs.x as isize, rhs.y as isize, rhs.z as isize);
        ((px - qx).pow(2) as f64 + (py - qy).pow(2) as f64 + (pz - qz).pow(2) as f64).sqrt()
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
impl Add<i64> for Vec3 {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        let Vec3 { x, y, z } = self;
        Self {
            x: x + rhs,
            y: y + rhs,
            z: z + rhs,
        }
    }
}
impl Div<i64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        let Vec3 { x, y, z } = self;
        Self {
            x: x / rhs,
            y: y / rhs,
            z: z / rhs,
        }
    }
}
impl Add<Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
