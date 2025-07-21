use std::collections::HashMap;

use math::{Vec2, Vec3};
use noise::{NoiseFn, Perlin};

fn matrix3d<T: Default>(size: Vec3) -> Vec<Vec<Vec<T>>> {
    let mut z_vec = Vec::with_capacity(size.z);
    for _ in 0..size.z {
        let mut y_vec = Vec::with_capacity(size.y);
        for _ in 0..size.y {
            let mut x_vec = Vec::with_capacity(size.x);
            for _ in 0..size.x {
                x_vec.push(T::default());
            }
            y_vec.push(x_vec);
        }
        z_vec.push(y_vec);
    }
    z_vec
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Box {
    x1: usize,
    x2: usize,
    z1: usize,
    z2: usize,
}
impl Box {
    fn intersect(&self, rhs: Self) -> bool {
        self.x1 <= rhs.x2 && self.x2 >= rhs.x1 && self.z1 <= rhs.z2 && self.z2 >= rhs.z1
    }
}

pub struct Map {
    alive: Vec<Vec<Vec<Piece>>>,
    dead: Vec<Vec<Vec<Option<usize>>>>,
    size: Vec3,
}
impl Default for Map {
    fn default() -> Self {
        let size = Vec3 {
            x: 80,
            y: 40,
            z: 80,
        };

        let mut map = Map {
            alive: matrix3d(size),
            dead: matrix3d(size),
            size,
        };

        // map.gen_caves(Vec3::new(0, 0, 0), Vec3::new(20, 20, 20));
        map.gen_sparse_floor(0);
        // map.gen_sparse_floor(10);
        // map.gen_sparse_floor(20);

        map
    }
}
impl Map {
    fn gen_caves(&mut self, min: Vec3, max: Vec3) {
        let noise = Perlin::new(rand::random());
        for x in min.x.min(max.x)..min.x.max(max.x) {
            for y in min.y.min(max.y)..min.y.max(max.y) {
                for z in min.z.min(max.z)..min.z.max(max.z) {
                    let f = noise.get([x as f64 * 10.0, y as f64 * 10.0, z as f64 * 10.0]);
                    trace!("{f}");
                    if f > 0.5 {
                        self.set(Vec3::new(x, y, z), Piece::Ground);
                    }
                }
            }
        }
    }
    fn gen_sparse_floor(&mut self, y: usize) {
        let mut boxes: HashMap<usize, (Box, Vec<usize>)> = Default::default();
        for i in 0..6 {
            let x_size = rand::random_range(4..=16);
            let z_size = rand::random_range(4..=16);
            let x = rand::random_range(0..=self.size.x - x_size);
            let z = rand::random_range(0..=self.size.z - z_size);
            let b = Box {
                x1: x,
                x2: x + x_size,
                z1: z,
                z2: z + z_size,
            };
            let mut clean = true;
            for (a, _) in boxes.values() {
                if a.intersect(b) {
                    clean = false;
                    break;
                }
            }
            if clean {
                boxes.insert(i, (b, Vec::new()));
            }
        }
        for (b, _) in boxes.values() {
            for x in b.x1..b.x2 {
                for z in b.z1..b.z2 {
                    self.set(Vec3::new(x, y, z), Piece::Ground);
                }
            }
        }
        for (a, _) in boxes.values() {
            for (b, _) in boxes.values() {
                if a == b {
                    continue;
                }
                let a_middle = Vec2::new(((a.x2 - a.x1) / 2) + a.x1, ((a.z2 - a.z1) / 2) + a.z1);
                let b_middle = Vec2::new(((b.x2 - b.x1) / 2) + b.x1, ((b.z2 - b.z1) / 2) + b.z1);

                let middle = if rand::random_bool(0.5) {
                    Vec2::new(a_middle.x, b_middle.y)
                } else {
                    Vec2::new(b_middle.x, a_middle.y)
                };
                self.set(Vec3::new(middle.x, y, middle.y), Piece::Ground);
                for p in a_middle.y.min(b_middle.y)..a_middle.y.max(b_middle.y) {
                    self.set(Vec3::new(middle.x, y, p), Piece::Ground);
                }
                for p in a_middle.x.min(b_middle.x)..a_middle.x.max(b_middle.x) {
                    self.set(Vec3::new(p, y, middle.y), Piece::Ground);
                }
            }
        }
    }

    pub fn get_size(&self) -> Vec3 {
        self.size
    }
    pub fn get_ground_data(&self) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        let (mut x_list, mut y_list, mut z_list) = (Vec::new(), Vec::new(), Vec::new());
        for z in 0..self.size.z {
            for y in 0..self.size.y {
                for x in 0..self.size.x {
                    if let Some(Piece::Ground) = self.get(Vec3::new(x, y, z)) {
                        x_list.push(x);
                        y_list.push(y);
                        z_list.push(z);
                    }
                }
            }
        }

        (x_list, y_list, z_list)
    }
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

    #[allow(unused)]
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
    Ground,
}
