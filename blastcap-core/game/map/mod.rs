use math::Vec2;

#[derive(Default)]
pub struct Map {
    alive: [[Piece; 16]; 16],
    dead: [[Option<usize>; 16]; 16],
}
impl Map {
    pub fn get(&self, Vec2 { x, y }: Vec2) -> Option<Piece> {
        self.alive.get(y).and_then(|r| r.get(x).copied())
    }

    pub fn set(&mut self, Vec2 { x, y }: Vec2, value: Piece) {
        let Some(piece) = self.alive.get_mut(y).and_then(|r| r.get_mut(x)) else {
            return;
        };
        *piece = value;
    }

    pub fn dead_get(&self, Vec2 { x, y }: Vec2) -> Option<usize> {
        self.dead
            .get(y)
            .and_then(|r| r.get(x).copied().and_then(|r| r))
    }

    pub fn dead_set(&mut self, Vec2 { x, y }: Vec2, value: Option<usize>) {
        let Some(piece) = self.dead.get_mut(y).and_then(|r| r.get_mut(x)) else {
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
