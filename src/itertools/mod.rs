use std::{collections::HashMap, hash::Hash};

pub mod pairs;

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

    fn cycle_length(self) -> Option<usize>
    where
        Self::Item: Eq + Hash,
    {
        let mut seen: HashMap<Self::Item, usize> = HashMap::new();
        for (idx, el) in self.enumerate() {
            if let Some(first_idx) = seen.get(&el) {
                return Some(idx - first_idx);
            }
            seen.insert(el, idx);
        }
        None
    }

    fn distance_to_cycle(self) -> Option<usize>
    where
        Self::Item: Eq + Hash,
    {
        let mut seen: HashMap<Self::Item, usize> = HashMap::new();
        for (idx, el) in self.enumerate() {
            if let Some(first_idx) = seen.get(&el) {
                return Some(*first_idx);
            }
            seen.insert(el, idx);
        }
        None
    }

    fn pairs(self) -> pairs::Pairs<Self>
    where
        Self: Clone,
        Self::Item: Clone,
    {
        pairs::Pairs::new(self)
    }

    fn pairs_with_repeats(self) -> pairs::PairsWithRepeats<Self>
    where
        Self: Clone,
        Self::Item: Clone,
    {
        pairs::PairsWithRepeats::new(self)
    }
}

impl<T> Itertools for T where T: Iterator {}
