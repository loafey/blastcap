use math::{Vec2, Vec3};
use rand::random_bool;

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
        let size = Vec3 { x: 80, y: 2, z: 80 };

        let mut map = Map {
            alive: matrix3d(size),
            dead: matrix3d(size),
            size,
        };
        let mut boxes: Vec<Box> = Vec::new();
        for _ in 0..6 {
            let x_size = rand::random_range(4..=16);
            let z_size = rand::random_range(4..=16);
            let x = rand::random_range(0..=map.size.x - x_size);
            let z = rand::random_range(0..=map.size.z - z_size);
            let b = Box {
                x1: x,
                x2: x + x_size,
                z1: z,
                z2: z + z_size,
            };
            let mut clean = true;
            for a in &boxes {
                if a.intersect(b) {
                    clean = false;
                    break;
                }
            }
            if clean {
                boxes.push(b);
            }
        }
        for b in &boxes {
            for x in b.x1..b.x2 {
                for z in b.z1..b.z2 {
                    map.set(Vec3::new(x, 0, z), Piece::Ground);
                }
            }
        }
        for a in &boxes {
            for b in &boxes {
                if a == b {
                    continue;
                }
                let a_middle = Vec2::new(((a.x2 - a.x1) / 2) + a.x1, ((a.z2 - a.z1) / 2) + a.z1);
                let b_middle = Vec2::new(((b.x2 - b.x1) / 2) + b.x1, ((b.z2 - b.z1) / 2) + b.z1);
                // map.set(Vec3::new(a_middle.x, 0, a_middle.y), Piece::Empty);
                // map.set(Vec3::new(b_middle.x, 0, b_middle.y), Piece::Empty);

                let middle = if rand::random_bool(0.5) {
                    Vec2::new(a_middle.x, b_middle.y)
                } else {
                    Vec2::new(b_middle.x, a_middle.y)
                };
                map.set(Vec3::new(middle.x, 0, middle.y), Piece::Ground);
                for p in a_middle.y.min(b_middle.y)..a_middle.y.max(b_middle.y) {
                    map.set(Vec3::new(middle.x, 0, p), Piece::Ground);
                }
                for p in a_middle.x.min(b_middle.x)..a_middle.x.max(b_middle.x) {
                    map.set(Vec3::new(p, 0, middle.y), Piece::Ground);
                }
                // let mut i = 0.0;
                // while i < 1.0 {
                //     let lerp = a_middle.lerp(b_middle, i);
                //     map.set(Vec3::new(lerp.x, 0, lerp.y), Piece::Ground);
                //     i += 0.01;
                // }
            }
        }
        // for x in 0..size.x {
        //     for z in 0..size.z {
        //         map.set(Vec3::new(x, 0, z), Piece::Ground);
        //     }
        // }
        // for y in 0..size.y {
        //     for x in 0..(size.x - y) {
        //         for z in 0..(size.z - y) {
        //             map.set(Vec3::new(x, y, z), Piece::Ground);
        //         }
        //     }
        // }
        // for y in 0..size.y {
        //     for x in 0..size.x {
        //         for z in 0..size.z {
        //             if rand::random_bool(0.5) {
        //                 map.set(Vec3::new(x, y, z), Piece::Ground);
        //             }
        //         }
        //     }
        // }
        map
    }
}
impl Map {
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
