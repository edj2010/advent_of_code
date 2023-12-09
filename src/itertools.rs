use std::{collections::HashMap, hash::Hash};

pub trait Itertools: Iterator + Sized {
    fn value_counts(self) -> HashMap<Self::Item, u32>
    where
        Self::Item: Eq + Hash,
    {
        let mut counts = HashMap::new();
        self.for_each(|item| *(counts.entry(item).or_default()) += 1);
        counts
    }
}

impl<T> Itertools for T where T: Iterator {}
