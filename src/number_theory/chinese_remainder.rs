use super::bezout_coefficients;
use num::traits::{One, Zero};
use std::ops::{Div, Mul, Rem, SubAssign};

/// Chinese Remainder Theorem:
///
/// Finds N and modulus M such that
/// N + i*M = remainder_a (mod modulus_a)
/// N + j*M = remainder_b (mod modulus_b)
/// for all i,j
pub fn chinese_remainder_with_modulus<T>(
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
            ((remainder_a * modulus_b * coef_b + remainder_b * modulus_a * coef_a) / gcd),
            modulus,
        ))
    } else {
        None
    }
}

/// Chinese Remainder Theorem:
///
/// Finds N such that
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
    chinese_remainder_with_modulus(remainder_a, modulus_a, remainder_b, modulus_b)
        .map(|(remainder, _)| remainder)
}

/// Chinese Remainder Theorem Many:
///
/// Finds N and modulus M such that, given a list of pairs (r_i, m_i)
/// N + k*M = r_i (mod m_i)
/// for all i,k
pub fn chinese_remainder_many_with_modulus<T, I: Iterator<Item = (T, T)>>(iter: I) -> Option<(T, T)>
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
        o.and_then(|(r_a, m_a)| (chinese_remainder_with_modulus(r_a, m_a, r_b, m_b)))
    })
}

/// Chinese Remainder Theorem Many:
///
/// Finds N such that, given a list of pairs (r_i, m_i)
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
    chinese_remainder_many_with_modulus(iter).map(|(r_a, _)| r_a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_coprime_moduli() {
        assert_eq!(chinese_remainder(0, 3, 3, 4), Some(-9));
        assert_eq!(chinese_remainder(1, 3, 3, 4), Some(-5));
    }

    #[test]
    fn two_non_coprime_moduli() {
        assert_eq!(chinese_remainder(13, 25, 8, 10), Some(-12));
        assert_eq!(chinese_remainder(13, 25, 9, 10), None);
    }

    #[test]
    fn two_moduli_with_modulus() {
        assert_eq!(chinese_remainder_with_modulus(0, 3, 3, 4), Some((-9, 12)));
        assert_eq!(chinese_remainder_with_modulus(1, 3, 3, 4), Some((-5, 12)));
        assert_eq!(
            chinese_remainder_with_modulus(13, 25, 8, 10),
            Some((-12, 50))
        );
        assert_eq!(chinese_remainder_with_modulus(13, 25, 9, 10), None);
    }

    #[test]
    fn many_moduli() {
        assert_eq!(
            chinese_remainder_many([(2, 3), (3, 5), (2, 7)].into_iter()),
            Some(-82)
        );
        assert_eq!(
            chinese_remainder_many([(0, 3), (3, 4), (4, 5)].into_iter()),
            Some(-21)
        );
        assert_eq!(
            chinese_remainder_many([(3, 6), (3, 4), (4, 5)].into_iter()),
            Some(-21)
        );
        assert_eq!(
            chinese_remainder_many([(0, 6), (3, 4), (4, 5)].into_iter()),
            None
        );
    }

    #[test]
    fn many_moduli_with_modulus() {
        assert_eq!(
            chinese_remainder_many_with_modulus([(2, 3), (3, 5), (2, 7)].into_iter()),
            Some((-82, 105))
        );
        assert_eq!(
            chinese_remainder_many_with_modulus([(0, 3), (3, 4), (4, 5)].into_iter()),
            Some((-21, 60))
        );
        assert_eq!(
            chinese_remainder_many_with_modulus([(3, 6), (3, 4), (4, 5)].into_iter()),
            Some((-21, 60))
        );
        assert_eq!(
            chinese_remainder_many_with_modulus([(0, 6), (3, 4), (4, 5)].into_iter()),
            None
        );
    }
}
