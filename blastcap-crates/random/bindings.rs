use crate::Random;

#[unsafe(no_mangle)]
pub extern "C" fn __random_new(seed: u64) -> *mut Random {
    Box::leak(Box::new(Random::new(seed)))
}
/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __random_drop(random: *mut Random) {
    _ = unsafe { Box::from_raw(random) };
}
macro_rules! random_int {
    ($($y:ty),+) => {$(pastey::paste! {
        /// # Safety
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn [<__random_gen_ $y>] (random: *mut Random) -> $y {
            let random = unsafe { &mut *random as &mut Random };
            random.get()
        }

        /// # Safety
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn [<__random_range_ $y>] (random: *mut Random, start: $y, end: $y) -> $y {
            let random = unsafe { &mut *random as &mut Random };
            random.get_range(start..end)
        }
    })+};
}
random_int!(u8, u16, u32, u64, i8, i16, i32, i64);

macro_rules! random_float {
    ($($y:ty),+) => {$(pastey::paste! {
        /// # Safety
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn [<__random_gen_ $y>] (random: *mut Random) -> $y {
            let random = unsafe { &mut *random as &mut Random };
            random.get_float()
        }

        /// # Safety
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn [<__random_range_ $y>] (random: *mut Random, start: $y, end: $y) -> $y {
            let random = unsafe { &mut *random as &mut Random };
            random.get_range_float(start..end)
        }
    })+};
}
random_float!(f32, f64);
