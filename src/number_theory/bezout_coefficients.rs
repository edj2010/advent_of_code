use num::traits::{One, Zero};
use std::ops::{Div, Mul, RemAssign, SubAssign};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct BezoutCoefficients<T> {
    r: T,
    s: T,
    t: T,
}

impl<T> BezoutCoefficients<T> {
    fn init(r: T, s: T, t: T) -> Self {
        BezoutCoefficients { r, s, t }
    }

    fn is_terminal(&self) -> bool
    where
        T: Zero,
    {
        self.r.is_zero()
    }

    fn to_tuple(self) -> (T, T, T) {
        (self.r, self.s, self.t)
    }
}

impl<T> Mul<T> for BezoutCoefficients<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        BezoutCoefficients {
            r: self.r * rhs.clone(),
            s: self.s * rhs.clone(),
            t: self.t * rhs,
        }
    }
}

impl<T> RemAssign<BezoutCoefficients<T>> for BezoutCoefficients<T>
where
    T: Div<T, Output = T> + Mul<T, Output = T> + SubAssign<T> + Clone,
{
    fn rem_assign(&mut self, rhs: BezoutCoefficients<T>) {
        let q = self.r.clone() / rhs.r.clone();
        self.r -= rhs.r * q.clone();
        self.s -= rhs.s * q.clone();
        self.t -= rhs.t * q;
    }
}

fn extended_euclidean_algoritm<T>(
    a: BezoutCoefficients<T>,
    mut b: BezoutCoefficients<T>,
) -> BezoutCoefficients<T>
where
    T: Zero + Div<T, Output = T> + Mul<T, Output = T> + SubAssign<T> + Clone,
{
    if b.is_terminal() {
        a
    } else {
        b %= a.clone();
        extended_euclidean_algoritm(a, b)
    }
}

/// Returns GCD and the Bezout Coefficients for a and b
/// Specifically, returns r, s, t such that r = gcd(a, b) = s*a + t*b
pub fn bezout_coefficients<T>(a: T, b: T) -> (T, T, T)
where
    T: One + Zero + Div<T, Output = T> + Mul<T, Output = T> + SubAssign<T> + Clone,
{
    extended_euclidean_algoritm(
        BezoutCoefficients::init(a, T::one(), T::zero()),
        BezoutCoefficients::init(b, T::zero(), T::one()),
    )
    .to_tuple()
}
