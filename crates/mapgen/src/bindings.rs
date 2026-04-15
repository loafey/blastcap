use crate::{Piece, generate_map};
use data::types::GroundType;
use smol::channel;

#[repr(C)]
pub struct GenerateMapFuncs {
    pub spawn_block: extern "C" fn(i64, i64, i64, GroundType),
    pub done: extern "C" fn(),
}

#[unsafe(no_mangle)]
pub extern "C" fn __generate_map(seed: u64, funcs: GenerateMapFuncs) {
    let (rx, tx) = channel::unbounded();
    generate_map(seed, crate::Output::new(rx));
    std::thread::spawn(move || {
        while let Ok((p, piece)) = tx.recv_blocking() {
            match piece {
                Piece::Actor(_) => todo!(),
                Piece::Ground(gtype) => (funcs.spawn_block)(p.x, p.y, p.z, gtype),
            }
        }
        (funcs.done)()
    });
}
