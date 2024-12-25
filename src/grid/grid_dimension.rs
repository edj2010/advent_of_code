use super::{grid_point, iterators};
use std::{
    fmt::Display,
    iter::Step,
    ops::{Add, Mul, Sub},
};
/////////////
/// Grid Dimension
///
/// datastructure for identifying Grid Dimentions
/////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GridDimensions<T> {
    pub min_row: T,
    pub max_row: T,
    pub min_col: T,
    pub max_col: T,
}

impl<T> GridDimensions<T> {
    pub fn new(min_row: T, max_row: T, min_col: T, max_col: T) -> Self {
        GridDimensions {
            min_row,
            max_row,
            min_col,
            max_col,
        }
    }

    pub fn contains(&self, point: &grid_point::GridPoint<T>) -> bool
    where
        T: PartialOrd,
    {
        self.min_row <= point.row
            && point.row < self.max_row
            && self.min_col <= point.col
            && point.col < self.max_col
    }

    pub fn rows(self) -> T
    where
        T: Sub<T, Output = T> + Add<T, Output = T>,
    {
        self.max_row - self.min_row
    }

    pub fn cols(self) -> T
    where
        T: Sub<T, Output = T> + Add<T, Output = T>,
    {
        self.max_col - self.min_col
    }

    pub fn area(self) -> T
    where
        T: Sub<T, Output = T> + Add<T, Output = T> + Mul<T, Output = T> + Step,
    {
        (self.max_row - self.min_row) * (self.max_col - self.min_col)
    }
}

impl<T> GridDimensions<T>
where
    T: Step + Ord + Clone,
{
    pub fn of_points_inclusive(a: grid_point::GridPoint<T>, b: grid_point::GridPoint<T>) -> Self {
        GridDimensions {
            min_row: a.row.clone().min(b.row.clone()),
            max_row: Step::forward(a.row.max(b.row), 1),
            min_col: a.col.clone().min(b.col.clone()),
            max_col: Step::forward(a.col.max(b.col), 1),
        }
    }

    pub fn all_contained_points(self) -> iterators::GridDimensionIterator<T> {
        iterators::GridDimensionIterator::new(self)
    }

    pub fn grow_to_contain(self, point: grid_point::GridPoint<T>) -> Self {
        let min_row = self.min_row.min(point.row.clone());
        let min_col = self.min_col.min(point.col.clone());
        let max_row = self.max_row.max(Step::forward(point.row, 1));
        let max_col = self.max_col.max(Step::forward(point.col, 1));
        GridDimensions {
            min_row,
            min_col,
            max_row,
            max_col,
        }
    }
}

impl<T: Display> Display for GridDimensions<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rows: ({} - {}), cols: ({} - {})",
            self.min_row, self.max_row, self.min_col, self.max_col
        )
    }
}
