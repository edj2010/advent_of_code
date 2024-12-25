use super::{block, grid_dimension, grid_point, iterators};
use std::{
    collections::{hash_map, HashMap},
    iter::IntoIterator,
    ops::{Index, IndexMut},
};

////////////
/// Lattice
///
/// similar interface to grid but allows for more efficient storage of
/// sparse data and allows for negative indicies
////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lattice<T> {
    points: HashMap<grid_point::GridPoint<isize>, T>,
}

impl<T: Clone> Lattice<T> {
    pub fn empty() -> Self {
        Lattice {
            points: HashMap::new(),
        }
    }

    pub fn from<I: IntoIterator<Item = (grid_point::GridPoint<isize>, T)>>(iter: I) -> Self {
        let mut lattice = Self::empty();
        for (point, value) in iter.into_iter() {
            lattice.set(point, value);
        }
        lattice
    }
}

impl<T> IntoIterator for Lattice<T> {
    type IntoIter = iterators::IntoGridLikeIterator<
        isize,
        T,
        hash_map::IntoIter<grid_point::GridPoint<isize>, T>,
    >;
    type Item = (grid_point::GridPoint<isize>, T);

    fn into_iter(self) -> Self::IntoIter {
        iterators::IntoGridLikeIterator::new(self.points.into_iter())
    }
}

impl<T> Lattice<T> {
    pub fn contains(&self, point: grid_point::GridPoint<isize>) -> bool {
        self.get(point).is_some()
    }

    pub fn get(&self, point: grid_point::GridPoint<isize>) -> Option<&T> {
        self.points.get(&point)
    }

    pub fn get_mut(&mut self, point: grid_point::GridPoint<isize>) -> Option<&mut T> {
        self.points.get_mut(&point)
    }

    pub fn set(&mut self, point: grid_point::GridPoint<isize>, value: T) -> Option<T> {
        self.points.insert(point, value)
    }

    pub fn entry(&mut self, point: grid_point::GridPoint<isize>) -> LatticeEntry<'_, T> {
        LatticeEntry(self.points.entry(point))
    }

    pub fn bounding_box(&self) -> Option<grid_dimension::GridDimensions<isize>> {
        self.points
            .iter()
            .fold(None, |grid_dimension, (&point, _)| match grid_dimension {
                None => Some(grid_dimension::GridDimensions::of_points_inclusive(
                    point, point,
                )),
                Some(dimension) => Some(dimension.grow_to_contain(point)),
            })
    }

    pub fn intersects_block(&self, block: &block::Block<isize>) -> bool {
        block.iter().any(|p| self.contains(*p))
    }

    pub fn set_block(&mut self, block: block::Block<isize>, value: T)
    where
        T: Clone,
    {
        block.into_iter().for_each(|p| {
            self.set(p, value.clone());
        })
    }

    pub fn apply_block_with_default<F: Fn(&mut T, grid_point::GridPoint<isize>)>(
        &mut self,
        block: block::Block<isize>,
        f: F,
    ) where
        T: Clone + Default,
    {
        block
            .into_iter()
            .for_each(|p| f(self.entry(p).or_default(), p))
    }

    pub fn apply_block<F: Fn(&mut T, grid_point::GridPoint<isize>)>(
        &mut self,
        block: block::Block<isize>,
        f: F,
        default: T,
    ) where
        T: Clone + Default,
    {
        block
            .into_iter()
            .for_each(|p| f(self.entry(p).or_insert(default.clone()), p))
    }
}

impl<T> Index<grid_point::GridPoint<isize>> for Lattice<T> {
    type Output = T;

    fn index(&self, index: grid_point::GridPoint<isize>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<grid_point::GridPoint<isize>> for Lattice<T> {
    fn index_mut(&mut self, index: grid_point::GridPoint<isize>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: PartialEq> Lattice<T> {
    pub fn find(&self, val: &T) -> Option<grid_point::GridPoint<isize>> {
        self.points
            .iter()
            .find(|(_, v)| &val == v)
            .map(|(idx, _)| idx)
            .copied()
    }
}

////////////
/// LatticeEntry
///
/// Used for modifying lattice entries
////////////

pub struct LatticeEntry<'a, T: 'a>(hash_map::Entry<'a, grid_point::GridPoint<isize>, T>);

impl<'a, T> LatticeEntry<'a, T> {
    pub fn or_insert(self, default: T) -> &'a mut T {
        self.0.or_insert(default)
    }

    pub fn or_insert_with<F: FnOnce() -> T>(self, default: F) -> &'a mut T {
        self.0.or_insert_with(default)
    }

    pub fn or_insert_with_key<F: FnOnce(&grid_point::GridPoint<isize>) -> T>(
        self,
        default: F,
    ) -> &'a mut T {
        self.0.or_insert_with_key(default)
    }

    pub fn key(&self) -> &grid_point::GridPoint<isize> {
        self.0.key()
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut T),
    {
        LatticeEntry(self.0.and_modify(f))
    }
}

impl<'a, T: Default> LatticeEntry<'a, T> {
    pub fn or_default(self) -> &'a mut T {
        self.0.or_default()
    }
}
