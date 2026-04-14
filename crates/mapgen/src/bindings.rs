use crate::{Piece, generate_map};
use math::Vec3;
use smol::channel;

#[repr(C)]
pub struct GenerateMapFuncs {
    pub spawn_block: extern "C" fn(usize, usize, usize),
    pub done: extern "C" fn(),
}

#[unsafe(no_mangle)]
pub extern "C" fn __generate_map(seed: u64, funcs: GenerateMapFuncs, x: usize, y: usize, z: usize) {
    let (rx, tx) = channel::unbounded();
    generate_map(seed, rx, Vec3::new(x, y, z));
    std::thread::spawn(move || {
        while let Ok((p, piece)) = tx.recv_blocking() {
            match piece {
                Piece::Empty => todo!(),
                Piece::Actor(_) => todo!(),
                Piece::Ground => (funcs.spawn_block)(p.x, p.y, p.z),
            }
        }
        (funcs.done)()
    });
}
