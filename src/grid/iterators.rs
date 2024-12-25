use super::{grid_dimension, grid_point, GridPoint};
use std::{iter::Step, marker::PhantomData, ops::Add};

////////////
/// Grid Point Iterators
///
/// helpful iterators for stepping over the grid
////////////

pub struct GridPointIterator<T, S> {
    next: Option<grid_point::GridPoint<T>>,
    traverse_by: grid_point::GridPointDelta<S>,
    grid_dimensions: grid_dimension::GridDimensions<T>,
}

impl<T, S> Iterator for GridPointIterator<T, S>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    T: PartialOrd + Clone,
{
    type Item = grid_point::GridPoint<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.clone() {
            self.next = next
                .clone()
                .add_checked(self.traverse_by.clone(), &self.grid_dimensions);
            Some(next)
        } else {
            None
        }
    }
}

impl<T> grid_point::GridPoint<T> {
    pub fn traverse_by<S>(
        self,
        traverse_by: grid_point::GridPointDelta<S>,
        grid_dimensions: grid_dimension::GridDimensions<T>,
    ) -> GridPointIterator<T, S> {
        GridPointIterator {
            next: Some(self),
            traverse_by,
            grid_dimensions,
        }
    }
}

impl grid_point::GridPoint<usize> {
    pub fn traverse_to(self, other: Self) -> Option<GridPointIterator<usize, isize>> {
        Some(GridPointIterator {
            next: Some(self),
            traverse_by: other.sub(self)?.min_step(),
            grid_dimensions: grid_dimension::GridDimensions::<usize>::of_points_inclusive(
                self, other,
            ),
        })
    }
}

impl grid_point::GridPoint<isize> {
    pub fn traverse_to(self, other: Self) -> Option<GridPointIterator<isize, isize>> {
        Some(GridPointIterator {
            next: Some(self),
            traverse_by: other.sub(self)?.min_step(),
            grid_dimensions: grid_dimension::GridDimensions::<isize>::of_points_inclusive(
                self, other,
            ),
        })
    }
}

////////////
/// GridLikeIterator
///
/// all the grid like objects have similarly shaped into iterators, so making a common type
////////////

pub struct GridLikeIterator<
    'a,
    T: 'a,
    I: Iterator<Item = (&'a grid_point::GridPoint<isize>, &'a T)>,
>(I);

impl<'a, T: 'a, I: Iterator<Item = (&'a grid_point::GridPoint<isize>, &'a T)>> Iterator
    for GridLikeIterator<'a, T, I>
{
    type Item = (&'a grid_point::GridPoint<isize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct GridLikeIteratorMut<
    'a,
    Idx: 'a,
    T: 'a,
    I: Iterator<Item = (&'a grid_point::GridPoint<Idx>, &'a mut T)>,
>(I);

impl<'a, Idx: 'a, T: 'a, I: Iterator<Item = (&'a grid_point::GridPoint<Idx>, &'a mut T)>> Iterator
    for GridLikeIteratorMut<'a, Idx, T, I>
{
    type Item = (&'a grid_point::GridPoint<Idx>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct IntoGridLikeIterator<Idx, T, I: Iterator<Item = (grid_point::GridPoint<Idx>, T)>>(
    I,
    PhantomData<Idx>,
);

impl<Idx, T, I: Iterator<Item = (grid_point::GridPoint<Idx>, T)>> IntoGridLikeIterator<Idx, T, I> {
    pub fn new(iter: I) -> Self {
        IntoGridLikeIterator(iter, PhantomData)
    }
}

impl<Idx, T, I: Iterator<Item = (grid_point::GridPoint<Idx>, T)>> Iterator
    for IntoGridLikeIterator<Idx, T, I>
{
    type Item = (grid_point::GridPoint<Idx>, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/////////////
/// Grid Dimension Iterator
///
/// iterator over all gridpoints within a specified dimension
/////////////

pub struct GridDimensionIterator<T: Step + Ord + Clone> {
    current_row: T,
    current_col: T,
    dimension: grid_dimension::GridDimensions<T>,
}

impl<T: Step + Ord + Clone> GridDimensionIterator<T> {
    pub fn new(dimension: grid_dimension::GridDimensions<T>) -> Self {
        GridDimensionIterator {
            current_row: dimension.min_row.clone(),
            current_col: dimension.min_col.clone(),
            dimension,
        }
    }
}

impl<T: Step + Ord + Clone> Iterator for GridDimensionIterator<T> {
    type Item = grid_point::GridPoint<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.dimension.max_row {
            return None;
        }
        let ret = grid_point::GridPoint::new(self.current_row.clone(), self.current_col.clone());

        self.current_col = Step::forward(self.current_col.clone(), 1);
        if self.current_col >= self.dimension.max_col {
            self.current_col = self.dimension.min_col.clone();
            self.current_row = Step::forward(self.current_row.clone(), 1);
        }
        Some(ret)
    }
}

//////////////
/// Sparse Grid Index Iterator
///
/// used for lattice and block iterators
//////////////

pub struct SpareGridIndexIterator<'a, T: 'a, I: Iterator<Item = &'a GridPoint<T>>>(I);

impl<'a, T: 'a, I: Iterator<Item = &'a GridPoint<T>>> SpareGridIndexIterator<'a, T, I> {
    pub fn new(iter: I) -> Self {
        SpareGridIndexIterator(iter)
    }
}

impl<'a, T: 'a, I: Iterator<Item = &'a GridPoint<T>>> Iterator
    for SpareGridIndexIterator<'a, T, I>
{
    type Item = &'a grid_point::GridPoint<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct SpareGridIndexIteratorMut<'a, T: 'a, I: Iterator<Item = &'a mut GridPoint<T>>>(I);

impl<'a, T: 'a, I: Iterator<Item = &'a mut GridPoint<T>>> SpareGridIndexIteratorMut<'a, T, I> {
    pub fn new(iter: I) -> Self {
        SpareGridIndexIteratorMut(iter)
    }
}

impl<'a, T: 'a, I: Iterator<Item = &'a mut GridPoint<T>>> Iterator
    for SpareGridIndexIteratorMut<'a, T, I>
{
    type Item = &'a mut grid_point::GridPoint<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct IntoSpareGridIndexIterator<T, I: Iterator<Item = GridPoint<T>>>(I);

impl<T, I: Iterator<Item = GridPoint<T>>> IntoSpareGridIndexIterator<T, I> {
    pub fn new(iter: I) -> Self {
        IntoSpareGridIndexIterator(iter)
    }
}

impl<T, I: Iterator<Item = GridPoint<T>>> Iterator for IntoSpareGridIndexIterator<T, I> {
    type Item = grid_point::GridPoint<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
