use math::Vec3;

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

pub struct Map {
    alive: Vec<Vec<Vec<Piece>>>,
    dead: Vec<Vec<Vec<Option<usize>>>>,
    size: Vec3,
}
impl Default for Map {
    fn default() -> Self {
        let size = Vec3 {
            x: 16,
            y: 16,
            z: 16,
        };
        let mut map = Map {
            alive: matrix3d(size),
            dead: matrix3d(size),
            size,
        };
        for y in 0..size.y {
            for x in 0..(size.x - y) {
                for z in 0..(size.z - y) {
                    map.set(Vec3::new(x, y, z), Piece::Ground);
                }
            }
        }
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
