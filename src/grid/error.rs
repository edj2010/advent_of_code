use super::{grid_dimension, grid_point};
use std::{
    error::Error,
    fmt::{Debug, Display},
};

////////////
/// Index Out Of Bounds error
///
/// error type for trying to index a part of the grid that doesn't exist
////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IndexOutOfBoundsError<T> {
    dimensions: grid_dimension::GridDimensions<T>,
    attempted: grid_point::GridPoint<T>,
}

impl<T> IndexOutOfBoundsError<T> {
    #[inline]
    pub fn new(
        dimensions: grid_dimension::GridDimensions<T>,
        attempted: grid_point::GridPoint<T>,
    ) -> Self {
        IndexOutOfBoundsError {
            dimensions,
            attempted,
        }
    }

    #[inline]
    pub fn err<U>(
        dimensions: grid_dimension::GridDimensions<T>,
        attempted: grid_point::GridPoint<T>,
    ) -> IndexResult<U, T> {
        Result::Err(IndexOutOfBoundsError::new(dimensions, attempted))
    }
}

impl<T: Display> Display for IndexOutOfBoundsError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attempted to access {} in grid with dimensions {}",
            self.attempted, self.dimensions,
        )
    }
}

impl<T: Debug + Display> Error for IndexOutOfBoundsError<T> {}

pub type IndexResult<T, S> = Result<T, IndexOutOfBoundsError<S>>;
