use num::traits::Zero;
use std::ops::Rem;

mod bezout_coefficients;
pub mod chinese_remainder;
pub use bezout_coefficients::bezout_coefficients;

pub fn gcd<T>(a: T, b: T) -> T
where
    T: Zero + PartialOrd + Rem<Output = T> + Clone,
{
    if b == T::zero() {
        a
    } else if a < b {
        gcd(b, a)
    } else {
        gcd(a.clone(), b % a)
    }
}
