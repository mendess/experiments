#[doc(hidden)]
pub use condtype::CondType;

#[macro_export]
macro_rules! min_type {
    ($pivot:literal, $($numeric:ty),+) => {
        <$crate::fits_pivot!($pivot, $($numeric),*) as $crate::Min>::Output
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! fits_pivot {
    ($pivot:literal) => { $crate::NoTypeCanHoldValue };
    ($pivot:literal, $numeric:ty $(, $rest:ty)*) => {
        $crate::CondType<
          {
              (<$numeric>::MIN as i128) <= ($pivot as i128) &&
                  ($pivot as i128) <= (<$numeric>::MAX as i128)
          },
          ($numeric, $crate::fits_pivot!($pivot $(, $rest)*)),
          $crate::fits_pivot!($pivot $(, $rest)*),
        >
    };
}

#[doc(hidden)]
pub trait Min {
    type Output;
}

pub struct NoTypeCanHoldValue(());

macro_rules! impl_min {
    ($($a:tt < $($b:tt),*);*$(;)?) => {
        $(
            $(
                impl $crate::Min for ($a, $b) {
                    type Output = $a;
                }

                impl $crate::Min for ($b, $a) {
                    type Output = $a;
                }
            )*

            impl $crate::Min for ($a, $a) {
                type Output = $a;
            }
        )*
    };
}

impl_min! {
    i8 < u16, u32, u64, u128, i16, i32, i64, i128;
    u8 < u16, u32, u64, u128, i16, i32, i64, i128;
    u16 < u32, u64, u128, i32, i64, i128;
    i16 < u32, u64, u128, i32, i64, i128;
    u32 < u64, u128, i64, i128;
    i32 < u64, u128, i64, i128;
    u64 < u128, i128;
    i64 < u128, i128;
}

impl<H, T> Min for (H, T)
where
    T: Min,
    (H, T::Output): Min,
{
    type Output = <(H, T::Output) as Min>::Output;
}

impl<H> Min for (H, NoTypeCanHoldValue) {
    type Output = H;
}
