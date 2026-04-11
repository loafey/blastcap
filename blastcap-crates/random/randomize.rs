pub(crate) trait Randomize:
    Sized
    + Copy
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Rem<Output = Self>
{
    fn rem_e(self, rhs: Self) -> Self;
}
macro_rules! randomize {
    ($($y:ty),+) => {
        $( impl Randomize for $y {
            fn rem_e(self, rhs: Self) -> Self {
                self.rem_euclid(rhs)
            }
        })+
    };
}
randomize!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
pub(crate) trait RandomizeFloat:
    Sized
    + Copy
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Rem<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
{
    const DIV: Self;
    fn from_u64(data: u64) -> Self;
}
macro_rules! randomize_float {
    ($($y:ty),+) => {
        $( impl RandomizeFloat for $y {
            const DIV: Self = const { u64::MAX as $y };
            fn from_u64(data: u64) -> $y {
                data as $y
            }
        })+
    };
}
randomize_float!(f32, f64);
