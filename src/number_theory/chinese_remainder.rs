use super::bezout_coefficients;
use num::traits::{One, Zero};
use std::ops::{Div, Mul, Rem, SubAssign};

fn chinese_remainder_inner<T>(
    remainder_a: T,
    modulus_a: T,
    remainder_b: T,
    modulus_b: T,
) -> Option<(T, T)>
where
    T: Clone
        + One
        + Zero
        + Div<T, Output = T>
        + Mul<T, Output = T>
        + SubAssign<T>
        + PartialEq
        + Rem<T, Output = T>,
{
    let (gcd, coef_a, coef_b) = bezout_coefficients(modulus_a.clone(), modulus_b.clone());
    if gcd.is_one() {
        let modulus = modulus_a.clone() * modulus_b.clone();
        Some((
            (remainder_a * modulus_b * coef_b + remainder_b * modulus_a * coef_a) % modulus.clone(),
            modulus,
        ))
    } else if remainder_a.clone() % gcd.clone() == remainder_b.clone() % gcd.clone() {
        let modulus = modulus_a.clone() * modulus_b.clone() / gcd.clone();
        Some((
            ((remainder_a * modulus_b * coef_b + remainder_b * modulus_a * coef_a) / gcd) % modulus.clone(),
            modulus,
        ))
    } else {
        None
    }
}

/// Chinese Remainder Theorem:
///
/// Finds the unique smallest N such that
/// N = remainder_a (mod modulus_a)
/// N = remainder_b (mod modulus_b)
pub fn chinese_remainder<T>(remainder_a: T, modulus_a: T, remainder_b: T, modulus_b: T) -> Option<T>
where
    T: Clone
        + One
        + Zero
        + Div<T, Output = T>
        + Mul<T, Output = T>
        + SubAssign<T>
        + PartialEq
        + Rem<T, Output = T>,
{
    chinese_remainder_inner(remainder_a, modulus_a, remainder_b, modulus_b)
        .map(|(remainder, _)| remainder)
}

/// Chinese Remainder Theorem Many:
///
/// Finds the unique smallest N such that, given a list of pairs (r_i, m_i)
/// N = r_i (mod m_i)
/// for all i
pub fn chinese_remainder_many<T, I: Iterator<Item = (T, T)>>(iter: I) -> Option<T>
where
    T: Clone
        + One
        + Zero
        + Div<T, Output = T>
        + Mul<T, Output = T>
        + SubAssign<T>
        + PartialEq
        + Rem<T, Output = T>,
{
    iter.fold(Some((T::zero(), T::one())), |o, (r_b, m_b)| {
        o.and_then(|(r_a, m_a)| (chinese_remainder_inner(r_a, m_a, r_b, m_b)))
    })
    .map(|(r_a, _)| r_a)
}
