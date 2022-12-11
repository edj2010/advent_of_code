use std::ops::{Add, BitAnd, BitOr, BitXor, Index, IndexMut, Sub};

#[derive(Debug)]
pub struct Vector<const SIZE: usize, T> {
    data: [T; SIZE],
}

impl<const SIZE: usize, T> Vector<SIZE, T> {
    pub fn of_list(l: Vec<T>) -> Result<Self, Vec<T>> {
        Ok(Vector {
            data: l.try_into()?,
        })
    }

    pub fn join<F: Fn(T, T) -> T>(self, other: Self, f: F) -> Self {
        Vector {
            data: self
                .data
                .into_iter()
                .zip(other.data.into_iter())
                .map(|(a, b)| f(a, b))
                .collect::<Vec<T>>()
                .try_into()
                .ok()
                .unwrap(),
        }
    }
}

impl<const SIZE: usize, T> Vector<SIZE, T>
where
    T: Copy,
{
    pub fn constant(t: T) -> Self {
        Vector { data: [t; SIZE] }
    }
}

impl<const SIZE: usize, T> Index<usize> for Vector<SIZE, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const SIZE: usize, T> IndexMut<usize> for Vector<SIZE, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const SIZE: usize, T> Clone for Vector<SIZE, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Vector {
            data: self.data.clone(),
        }
    }
}

impl<const SIZE: usize, T> Copy for Vector<SIZE, T> where T: Copy {}

impl<const SIZE: usize, T> Add<Self> for Vector<SIZE, T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.join(rhs, |a, b| a + b)
    }
}

impl<const SIZE: usize, T> Sub<Self> for Vector<SIZE, T>
where
    T: Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.join(rhs, |a, b| a - b)
    }
}

impl<const SIZE: usize, T> BitAnd<Self> for Vector<SIZE, T>
where
    T: BitAnd<T, Output = T>,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.join(rhs, |a, b| a & b)
    }
}

impl<const SIZE: usize, T> BitOr<Self> for Vector<SIZE, T>
where
    T: BitOr<T, Output = T>,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.join(rhs, |a, b| a | b)
    }
}

impl<const SIZE: usize, T> BitXor<Self> for Vector<SIZE, T>
where
    T: BitXor<T, Output = T>,
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.join(rhs, |a, b| a ^ b)
    }
}
