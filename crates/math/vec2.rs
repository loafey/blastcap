#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
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
}
