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

    fn contains(mut self, item: &Self::Item) -> bool
    where
        Self::Item: PartialEq,
    {
        self.find(|i| i == item).is_some()
    }
}

impl<T> Itertools for T where T: Iterator {}
