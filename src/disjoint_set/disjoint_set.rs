use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

pub trait DisjointSet<T: Copy> {
    fn is_connected(&mut self, a: T, b: T) -> Option<bool>;
    fn union(&mut self, a: T, b: T) -> Option<bool>;
    fn size(&self) -> usize;
    fn all_sets(&mut self) -> Vec<Vec<T>>;
}

#[derive(Debug, PartialEq, Eq)]
struct Node {
    parent: usize,
    rank: u32,
}

impl Node {
    fn init(idx: usize) -> Self {
        Node {
            parent: idx,
            rank: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DisjointUsizeSet {
    nodes: Vec<Node>,
}

impl DisjointUsizeSet {
    pub fn empty() -> Self {
        DisjointUsizeSet { nodes: vec![] }
    }

    pub fn init(size: usize) -> Self {
        DisjointUsizeSet {
            nodes: (0..size).map(|idx| Node::init(idx)).collect(),
        }
    }

    pub fn add_new(&mut self) -> usize {
        self.nodes.push(Node::init(self.nodes.len()));
        return self.nodes.len() - 1;
    }

    fn find_root(&mut self, key: usize) -> Option<usize> {
        let mut current = key;
        while current != self.nodes.get(current)?.parent {
            self.nodes.get_mut(current)?.parent =
                self.nodes.get(self.nodes.get(current)?.parent)?.parent;
            current = self.nodes.get(current)?.parent;
        }
        return Some(current);
    }
}

impl DisjointSet<usize> for DisjointUsizeSet {
    fn is_connected(&mut self, a: usize, b: usize) -> Option<bool> {
        Some(self.find_root(a)? == self.find_root(b)?)
    }

    fn union(&mut self, a: usize, b: usize) -> Option<bool> {
        let root_a = self.find_root(a)?;
        let root_b = self.find_root(b)?;
        if root_a == root_b {
            return Some(false);
        }
        match self.nodes[root_a].rank.cmp(&self.nodes[root_b].rank) {
            std::cmp::Ordering::Less => self.nodes[root_a].parent = root_b,
            std::cmp::Ordering::Greater => self.nodes[root_b].parent = root_a,
            std::cmp::Ordering::Equal => {
                self.nodes[root_a].parent = root_b;
                self.nodes[root_a].rank += 1;
            }
        }
        Some(true)
    }

    fn size(&self) -> usize {
        self.nodes.len()
    }

    fn all_sets(&mut self) -> Vec<Vec<usize>> {
        let mut sets_by_parent_key: HashMap<usize, Vec<usize>> = HashMap::new();
        for el in 0..self.size() {
            sets_by_parent_key
                .entry(self.nodes[el].parent)
                .or_default()
                .push(el);
        }
        sets_by_parent_key.into_values().collect()
    }
}

pub struct DisjointBTreeSet<T>
where
    T: Ord + Copy,
{
    inner: DisjointUsizeSet,
    key_map: BTreeMap<T, usize>,
}

impl<T> DisjointBTreeSet<T>
where
    T: Ord + Copy,
{
    pub fn empty() -> Self {
        DisjointBTreeSet {
            inner: DisjointUsizeSet::empty(),
            key_map: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> bool {
        if self.key_map.contains_key(&item) {
            return false;
        }

        self.key_map.insert(item, self.inner.add_new());
        true
    }
}

impl<T> DisjointSet<T> for DisjointBTreeSet<T>
where
    T: Ord + Copy,
{
    fn is_connected(&mut self, a: T, b: T) -> Option<bool> {
        self.inner
            .is_connected(*self.key_map.get(&a)?, *self.key_map.get(&b)?)
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn union(&mut self, a: T, b: T) -> Option<bool> {
        self.inner
            .union(*self.key_map.get(&a)?, *self.key_map.get(&b)?)
    }

    fn all_sets(&mut self) -> Vec<Vec<T>> {
        let mut sets_by_parent_key: HashMap<usize, Vec<T>> = HashMap::new();
        for (&el, &key) in self.key_map.iter() {
            sets_by_parent_key
                .entry(self.inner.nodes[key].parent)
                .or_default()
                .push(el);
        }
        sets_by_parent_key.into_values().collect()
    }
}

pub struct DisjointHashSet<T>
where
    T: Hash + Eq + Copy,
{
    inner: DisjointUsizeSet,
    key_map: HashMap<T, usize>,
}

impl<T> DisjointHashSet<T>
where
    T: Hash + Eq + Copy,
{
    pub fn empty() -> Self {
        DisjointHashSet {
            inner: DisjointUsizeSet::empty(),
            key_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> bool {
        if self.key_map.contains_key(&item) {
            return false;
        }

        self.key_map.insert(item, self.inner.add_new());
        true
    }
}

impl<T> DisjointSet<T> for DisjointHashSet<T>
where
    T: Hash + Eq + Copy,
{
    fn is_connected(&mut self, a: T, b: T) -> Option<bool> {
        self.inner
            .is_connected(*self.key_map.get(&a)?, *self.key_map.get(&b)?)
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn union(&mut self, a: T, b: T) -> Option<bool> {
        self.inner
            .union(*self.key_map.get(&a)?, *self.key_map.get(&b)?)
    }

    fn all_sets(&mut self) -> Vec<Vec<T>> {
        let mut sets_by_parent_key: HashMap<usize, Vec<T>> = HashMap::new();
        for (&el, &key) in self.key_map.iter() {
            let root = self.inner.find_root(key).unwrap();
            sets_by_parent_key.entry(root).or_default().push(el);
        }
        sets_by_parent_key.into_values().collect()
    }
}

#[cfg(test)]
mod tests {

    mod disjoint_set_usize_tests {
        use super::super::*;
        use std::collections::BTreeSet;

        #[test]
        fn test_empty() {
            assert_eq!(DisjointUsizeSet::empty().size(), 0);
        }

        #[test]
        fn test_init() {
            let mut new = DisjointUsizeSet::init(10);
            assert_eq!(new.size(), 10);
            for el in 0..10 {
                assert_eq!(new.find_root(el), Some(el));
            }
        }

        #[test]
        fn test_insert() {
            let a = DisjointUsizeSet::init(10);
            let mut b = DisjointUsizeSet::empty();
            for _ in 0..10 {
                b.add_new();
            }

            assert_eq!(a, b);
        }

        #[test]
        fn two_sets() {
            let mut new = DisjointUsizeSet::init(10);
            for el in 2..10 {
                new.union(el, el - 2);
            }
            for a in 0..10 {
                for b in 0..10 {
                    assert_eq!(new.is_connected(a, b), Some(a % 2 == b % 2));
                }
            }
            let mut all_sets = new.all_sets();
            all_sets.sort();
            assert_eq!(all_sets.len(), 2);
            assert_eq!(
                BTreeSet::from_iter(all_sets[0].iter().copied()),
                BTreeSet::from_iter(vec![0, 2, 4, 6, 8].into_iter())
            );
            assert_eq!(
                BTreeSet::from_iter(all_sets[1].iter().copied()),
                BTreeSet::from_iter(vec![1, 3, 5, 7, 9].into_iter())
            );
        }
    }

    mod disjoint_set_btree_tests {
        use super::super::*;
        use std::collections::BTreeSet;

        #[test]
        fn test_empty() {
            assert_eq!(DisjointBTreeSet::<usize>::empty().size(), 0);
        }

        #[test]
        fn test_init() {
            let mut new: DisjointBTreeSet<u32> = DisjointBTreeSet::empty();
            for el in (0..100).step_by(10) {
                new.insert(el);
            }
            assert_eq!(new.size(), 10);
            for a in (0..100).step_by(10) {
                for b in (0..100).step_by(10) {
                    assert_eq!(new.is_connected(a, b), Some(a == b));
                }
            }
        }

        #[test]
        fn two_sets() {
            let mut new: DisjointBTreeSet<u32> = DisjointBTreeSet::empty();
            for el in (0..100).step_by(10) {
                new.insert(el);
            }
            for el in (20..100).step_by(10) {
                new.union(el, el - 20);
            }
            for a in (0..100).step_by(10) {
                for b in (0..100).step_by(10) {
                    assert_eq!(new.is_connected(a, b), Some(a % 20 == b % 20));
                }
            }
            let mut all_sets = new.all_sets();
            all_sets.sort();
            assert_eq!(all_sets.len(), 2);
            assert_eq!(
                BTreeSet::from_iter(all_sets[0].iter().copied()),
                BTreeSet::from_iter(vec![0, 20, 40, 60, 80].into_iter())
            );
            assert_eq!(
                BTreeSet::from_iter(all_sets[1].iter().copied()),
                BTreeSet::from_iter(vec![10, 30, 50, 70, 90].into_iter())
            );
        }
    }
    mod disjoint_set_hash_tests {
        use std::collections::BTreeSet;

        use super::super::*;

        #[test]
        fn test_empty() {
            assert_eq!(DisjointHashSet::<usize>::empty().size(), 0);
        }

        #[test]
        fn test_init() {
            let mut new: DisjointHashSet<u32> = DisjointHashSet::empty();
            for el in (0..100).step_by(10) {
                new.insert(el);
            }
            assert_eq!(new.size(), 10);
            for a in (0..100).step_by(10) {
                for b in (0..100).step_by(10) {
                    assert_eq!(new.is_connected(a, b), Some(a == b));
                }
            }
        }

        #[test]
        fn two_sets() {
            let mut new: DisjointHashSet<u32> = DisjointHashSet::empty();
            for el in (0..100).step_by(10) {
                new.insert(el);
            }
            for el in (20..100).step_by(10) {
                new.union(el, el - 20);
            }
            for a in (0..100).step_by(10) {
                for b in (0..100).step_by(10) {
                    assert_eq!(new.is_connected(a, b), Some(a % 20 == b % 20));
                }
            }
            let mut all_sets = new.all_sets();
            all_sets.sort();
            assert_eq!(all_sets.len(), 2);
            assert_eq!(
                BTreeSet::from_iter(all_sets[0].iter().copied()),
                BTreeSet::from_iter(vec![0, 20, 40, 60, 80].into_iter())
            );
            assert_eq!(
                BTreeSet::from_iter(all_sets[1].iter().copied()),
                BTreeSet::from_iter(vec![10, 30, 50, 70, 90].into_iter())
            );
        }
    }
}
