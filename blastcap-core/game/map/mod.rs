use math::Vec3;

#[derive(Default)]
pub struct Map {
    alive: [[[Piece; 16]; 16]; 16],
    dead: [[[Option<usize>; 16]; 16]; 16],
}
impl Map {
    pub fn get(&self, Vec3 { x, y, z }: Vec3) -> Option<Piece> {
        self.alive
            .get(z)
            .and_then(|r| r.get(y).and_then(|r| r.get(x).copied()))
    }

    pub fn set(&mut self, Vec3 { x, y, z }: Vec3, value: Piece) {
        let Some(piece) = self
            .alive
            .get_mut(z)
            .and_then(|r| r.get_mut(y).and_then(|r| r.get_mut(x)))
        else {
            return;
        };
        *piece = value;
    }

    pub fn dead_get(&self, Vec3 { x, y, z }: Vec3) -> Option<usize> {
        self.dead
            .get(z)
            .and_then(|r| r.get(y).and_then(|r| r.get(x).copied()))
            .and_then(|r| r)
    }

    pub fn dead_set(&mut self, Vec3 { x, y, z }: Vec3, value: Option<usize>) {
        let Some(piece) = self
            .dead
            .get_mut(z)
            .and_then(|r| r.get_mut(y).and_then(|r| r.get_mut(x)))
        else {
            return;
        };
        *piece = value;
    }
}
#[derive(Default, Debug, Clone, Copy)]
pub enum Piece {
    #[default]
    Empty,
    Actor(usize),
}
