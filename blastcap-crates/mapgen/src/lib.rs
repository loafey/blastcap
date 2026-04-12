use math::{Vec2, Vec3};
use random::Random;
use smol::channel;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Copy)]
pub enum Piece {
    #[default]
    Empty,
    Actor(usize),
    Ground,
}

pub type Output = channel::Sender<(Vec3, Piece)>;

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

fn gen_sparse_floor(rand: &mut Random, y: usize, floor_amount: usize, size: Vec3, spawn: Output) {
    let set = |p| spawn.send_blocking(p).unwrap();
    let mut boxes: HashMap<usize, (Box, Vec<usize>)> = Default::default();
    for i in 0..floor_amount {
        let x_size = rand.get_range(4..17);
        let z_size = rand.get_range(4..17);
        let x = rand.get_range(0..(1 + size.x - x_size));
        let z = rand.get_range(0..(1 + size.z - z_size));
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
                set((Vec3::new(x, y, z), Piece::Ground));
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

            let middle = if rand.bool() {
                Vec2::new(a_middle.x, b_middle.y)
            } else {
                Vec2::new(b_middle.x, a_middle.y)
            };
            set((Vec3::new(middle.x, y, middle.y), Piece::Ground));
            for p in a_middle.y.min(b_middle.y)..a_middle.y.max(b_middle.y) {
                set((Vec3::new(middle.x, y, p), Piece::Ground));
            }
            for p in a_middle.x.min(b_middle.x)..a_middle.x.max(b_middle.x) {
                set((Vec3::new(p, y, middle.y), Piece::Ground));
            }
        }
    }
}

pub fn generate_map(seed: u64, spawn: Output, size: Vec3) {
    std::thread::spawn(move || {
        let mut random = Random::new(seed);
        gen_sparse_floor(&mut random, 0, 3, size, spawn);
    });
}
