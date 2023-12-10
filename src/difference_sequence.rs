use num::traits::Zero;
use std::ops::{AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferenceSequence<T> {
    state: Vec<T>,
}

pub fn difference<T>(v: &[T]) -> Vec<T>
where
    T: Sub<T, Output = T> + Clone,
{
    let mut difference = vec![];
    for idx in 1..v.len() {
        difference.push(v[idx].clone() - v[idx - 1].clone());
    }
    difference
}

impl<T> DifferenceSequence<T> {
    pub fn derive<I: IntoIterator<Item = T>>(v: I) -> Self
    where
        T: Zero + Sub<T, Output = T> + Clone,
    {
        let mut state: Vec<T> = vec![];
        let mut current: Vec<T> = v.into_iter().collect();
        while current.iter().any(|c| !c.is_zero()) {
            state.push(current[0].clone());
            current = difference(&current);
        }
        DifferenceSequence { state }
    }

    pub fn current_value(&self) -> T
    where
        T: Clone,
    {
        self.state[0].clone()
    }

    pub fn step_back(&mut self) -> T
    where
        T: SubAssign<T> + Clone,
    {
        for idx in (1..self.state.len()).rev() {
            let difference = self.state[idx].clone();
            self.state[idx - 1] -= difference;
        }
        self.state[0].clone()
    }
}

impl<T> FromIterator<T> for DifferenceSequence<T>
where
    T: Zero + Sub<T, Output = T> + Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        DifferenceSequence::derive(iter)
    }
}

impl<T> Iterator for DifferenceSequence<T>
where
    T: AddAssign<T> + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let to_return = Some(self.state[0].clone());
        for idx in 1..self.state.len() {
            let difference = self.state[idx].clone();
            self.state[idx - 1] += difference;
        }
        to_return
    }
}
