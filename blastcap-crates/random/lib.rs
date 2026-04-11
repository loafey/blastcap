use crate::randomize::{Randomize, RandomizeFloat};
use std::ops::Range;

pub const POOL_SIZE: usize = 1024;
const SOURCE: &[u8] = include_bytes!("rand.bin");

mod randomize;

/// this is not cryptographically secure, and should not be treated as such :)
/// slapped together for fun
pub struct Random {
    index: usize,
    inc: u32,
    state: u32,
    data: &'static [u8],
}
impl Random {
    pub fn new(seed: u64) -> Self {
        Self::with_pool(seed, SOURCE)
    }
    pub fn with_pool(seed: u64, data: &'static [u8]) -> Self {
        let b = seed.to_be_bytes();
        let index = u16::from_be_bytes([b[0], b[1]]) as usize % data.len();
        let inc = u16::from_be_bytes([b[2], b[3]]) as u32;
        let state = u32::from_be_bytes([b[4], b[5], b[6], b[7]]);
        Self {
            index,
            data,
            inc,
            state,
        }
    }

    pub fn get_u8(&mut self) -> u8 {
        let val = self.data[self.index] as u64;
        self.state = self.state.wrapping_add(self.inc);
        let state = self.state as u64;
        self.index = (self.index + 1) % self.data.len();
        ((val + state) % 255) as u8
    }

    // this only works for numbers so it shouuuuld be safe tihi
    #[allow(private_bounds, clippy::uninit_assumed_init)]
    pub fn get<T: Randomize>(&mut self) -> T {
        let mut data = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let ptr = unsafe {
            std::mem::transmute::<(&mut T, usize), &mut [u8]>((&mut data, size_of::<T>()))
        };
        (0..size_of::<T>()).for_each(|i| ptr[i] = self.get_u8());
        data
    }

    #[allow(private_bounds)]
    pub fn get_float<T: RandomizeFloat>(&mut self) -> T {
        T::from_u64(self.get::<u64>()) / T::DIV
    }

    #[allow(private_bounds)]
    pub fn get_range<T: Randomize>(&mut self, range: Range<T>) -> T {
        let bound = range.end - range.start;
        let value = self.get::<T>().rem_e(bound);
        value + range.start
    }

    #[allow(private_bounds)]
    pub fn get_range_float<T: RandomizeFloat>(&mut self, range: Range<T>) -> T {
        let bound = range.end - range.start;
        let value = self.get_float::<T>() * bound;
        value + range.start
    }
}
