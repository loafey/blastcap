use mapgen::Piece;
use math::Vec3;
use std::collections::HashMap;

pub struct Map {
    alive: HashMap<Vec3, Piece>,
    dead: HashMap<Vec3, usize>,
}
impl Map {
    pub fn gen_map(f: impl FnOnce(&mut Map)) -> Self {
        let mut map = Map {
            alive: Default::default(),
            dead: Default::default(),
        };
        f(&mut map);
        map
    }

    pub fn get_map_positions(&self) -> impl Iterator<Item = Vec3> {
        self.alive.keys().copied()
    }

    pub fn get(&self, pos: Vec3) -> Option<Piece> {
        self.alive.get(&pos).cloned()
    }

    pub fn set(&mut self, pos: Vec3, value: Piece) {
        self.alive.insert(pos, value);
    }

    pub fn remove(&mut self, pos: Vec3) -> Option<Piece> {
        self.alive.remove(&pos)
    }

    #[allow(unused)]
    pub fn dead_get(&self, pos: Vec3) -> Option<usize> {
        self.dead.get(&pos).copied()
    }

    pub fn dead_set(&mut self, pos: Vec3, value: usize) {
        self.dead.insert(pos, value);
    }
}
