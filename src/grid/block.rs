use std::{fmt::Debug, ops::Add};

////////////
/// Block
///
/// very small grid, designed to be moved around,
/// checked for intersection, and added to a lattice or grid
////////////
use super::{grid_dimension, grid_point, iterators};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<T>(Vec<grid_point::GridPoint<T>>);

impl<T> Block<T> {
    pub fn empty() -> Self {
        Block(Vec::new())
    }

    pub fn from<I: IntoIterator<Item = grid_point::GridPoint<T>>>(iter: I) -> Self {
        Block(iter.into_iter().collect())
    }

    pub fn add_point(&mut self, point: grid_point::GridPoint<T>) {
        self.0.push(point);
    }

    pub fn intersects(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.0.iter().any(|p| other.0.contains(p))
    }

    pub fn min_row(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.0.iter().map(|p| p.row()).min()
    }

    pub fn max_row(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.0.iter().map(|p| p.row()).max()
    }

    pub fn min_col(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.0.iter().map(|p| p.col()).min()
    }

    pub fn max_col(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.0.iter().map(|p| p.col()).max()
    }

    pub fn dimensions(&self) -> Option<grid_dimension::GridDimensions<&T>>
    where
        T: Ord,
    {
        Some(grid_dimension::GridDimensions {
            min_row: self.min_row()?,
            max_row: self.max_row()?,
            min_col: self.min_col()?,
            max_col: self.max_col()?,
        })
    }

    pub fn add_checked<S>(
        self,
        rhs: grid_point::GridPointDelta<S>,
        grid_dimensions: &grid_dimension::GridDimensions<T>,
    ) -> Option<Block<T>>
    where
        T: PartialOrd,
        S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    {
        Some(Block(
            self.0
                .into_iter()
                .map(|p| p.add_checked(rhs.clone(), grid_dimensions))
                .collect::<Option<Vec<grid_point::GridPoint<T>>>>()?,
        ))
    }

    pub fn iter<'a>(
        &'a self,
    ) -> iterators::SpareGridIndexIterator<'a, T, std::slice::Iter<'a, grid_point::GridPoint<T>>>
    {
        iterators::SpareGridIndexIterator::new(self.0.iter())
    }

    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> iterators::SpareGridIndexIteratorMut<
        'a,
        T,
        std::slice::IterMut<'a, grid_point::GridPoint<T>>,
    > {
        iterators::SpareGridIndexIteratorMut::new(self.0.iter_mut())
    }

    pub fn into_iter(
        self,
    ) -> iterators::IntoSpareGridIndexIterator<T, std::vec::IntoIter<grid_point::GridPoint<T>>>
    {
        iterators::IntoSpareGridIndexIterator::new(self.0.into_iter())
    }
}

impl<T, S> Add<grid_point::GridPointDelta<S>> for Block<T>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    <S as TryInto<T>>::Error: Debug,
    <S as TryFrom<T>>::Error: Debug,
{
    type Output = Block<T>;

    fn add(self, rhs: grid_point::GridPointDelta<S>) -> Self::Output {
        Block(
            self.0
                .into_iter()
                .map(|p| p + rhs.clone())
                .collect::<Vec<grid_point::GridPoint<T>>>(),
        )
    }
}
