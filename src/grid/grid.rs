use super::{constants, error, grid_dimension, grid_point, iterators};
use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

////////////
/// Grid
///
/// simple datastructure for holding a grid of values
////////////

#[derive(Clone, PartialEq, Eq)]
pub struct Grid<T> {
    dimensions: grid_dimension::GridDimensions<usize>,
    grid: Vec<T>,
}

impl<T: Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.dimensions.rows() {
            for col in 0..self.dimensions.cols() {
                write!(f, "{:?} ", self.get(grid_point::GridPoint::new(row, col)))?
            }
            write!(f, "\n")?
        }
        write!(f, "")
    }
}

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.dimensions.rows() {
            for col in 0..self.dimensions.cols() {
                write!(
                    f,
                    "{} ",
                    self.get(grid_point::GridPoint::new(row, col)).unwrap()
                )?
            }
            write!(f, "\n")?
        }
        write!(f, "")
    }
}

impl<T: Clone> Grid<T> {
    pub fn init(init: T, rows: usize, cols: usize) -> Self {
        let dimensions = grid_dimension::GridDimensions::new(0, rows, 0, cols);
        Grid {
            dimensions,
            grid: vec![init; rows * cols],
        }
    }

    pub fn from<I: IntoIterator<Item = T>>(v: I, rows: usize, cols: usize) -> Option<Self> {
        let dimensions = grid_dimension::GridDimensions::new(0, rows, 0, cols);
        let grid: Vec<T> = v.into_iter().collect::<Vec<T>>();
        if grid.len() == rows * cols {
            Some(Grid { dimensions, grid })
        } else {
            None
        }
    }

    pub fn of_list_of_lists<J: IntoIterator<Item = T>, I: IntoIterator<Item = J>>(
        v: I,
        rows: usize,
        cols: usize,
    ) -> Option<Self> {
        let dimensions = grid_dimension::GridDimensions::new(0, rows, 0, cols);

        let mut grid: Vec<T> = Vec::with_capacity(rows * cols);
        for (row_idx, row) in v.into_iter().enumerate() {
            grid.extend(row);
            if grid.len() != cols * (row_idx + 1) {
                return None;
            }
        }
        Some(Grid { dimensions, grid })
    }

    pub fn of_vec_of_vecs(v: Vec<Vec<T>>) -> Option<Self> {
        let rows = v.len();
        let cols = v[0].len();
        Self::of_list_of_lists(v, rows, cols)
    }
}

fn point_as_arr_idx(
    dimension: grid_dimension::GridDimensions<usize>,
    grid_point: grid_point::GridPoint<usize>,
) -> usize {
    grid_point.row() * dimension.cols() + grid_point.col()
}

fn point_of_arr_idx(
    dimension: grid_dimension::GridDimensions<usize>,
    arr_idx: usize,
) -> grid_point::GridPoint<usize> {
    grid_point::GridPoint::new(
        arr_idx.clone() / dimension.cols(),
        arr_idx % dimension.cols(),
    )
}

impl<T> Grid<T> {
    pub fn cols(&self) -> usize {
        self.dimensions.cols()
    }

    pub fn rows(&self) -> usize {
        self.dimensions.rows()
    }

    pub fn dimensions(&self) -> grid_dimension::GridDimensions<usize> {
        self.dimensions
    }

    pub fn iter_points(&self) -> iterators::GridDimensionIterator<usize> {
        self.dimensions.all_contained_points()
    }

    pub fn get(&self, point: grid_point::GridPoint<usize>) -> error::IndexResult<&T, usize> {
        if !self.dimensions.contains(&point) {
            return error::IndexOutOfBoundsError::err(self.dimensions, point);
        }
        self.grid
            .get(point_as_arr_idx(self.dimensions, point))
            .ok_or(error::IndexOutOfBoundsError::new(self.dimensions, point))
    }

    pub fn get_mut(
        &mut self,
        point: grid_point::GridPoint<usize>,
    ) -> error::IndexResult<&mut T, usize> {
        if !self.dimensions.contains(&point) {
            return error::IndexOutOfBoundsError::err(self.dimensions, point);
        }
        let index = point_as_arr_idx(self.dimensions, point);
        self.grid
            .get_mut(index)
            .ok_or(error::IndexOutOfBoundsError::new(self.dimensions, point))
    }

    pub fn set(
        &mut self,
        point: grid_point::GridPoint<usize>,
        value: T,
    ) -> error::IndexResult<(), usize> {
        if !self.dimensions.contains(&point) {
            return error::IndexOutOfBoundsError::err(self.dimensions, point);
        }
        let index = point_as_arr_idx(self.dimensions, point);
        *self
            .grid
            .get_mut(index)
            .ok_or(error::IndexOutOfBoundsError::new(self.dimensions, point))? = value;
        Ok(())
    }

    pub fn all_rows(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.grid.chunks(self.cols()).map(|s| s.into()).collect()
    }

    pub fn all_cols(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        grid_point::GridPoint::new(0, 0)
            .traverse_by(constants::EAST, self.dimensions())
            .map(|col_start| {
                col_start
                    .traverse_by(constants::SOUTH, self.dimensions())
                    .map(|point| self.get(point).unwrap().clone())
                    .collect()
            })
            .collect()
    }
}

impl<T> Index<grid_point::GridPoint<usize>> for Grid<T> {
    type Output = T;

    fn index(&self, index: grid_point::GridPoint<usize>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<grid_point::GridPoint<usize>> for Grid<T> {
    fn index_mut(&mut self, index: grid_point::GridPoint<usize>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: PartialEq> Grid<T> {
    pub fn find(&self, val: &T) -> Option<grid_point::GridPoint<usize>> {
        self.grid
            .iter()
            .enumerate()
            .find(|(_, v)| &val == v)
            .map(|(idx, _)| point_of_arr_idx(self.dimensions, idx))
    }
}

impl<T> IntoIterator for Grid<T> {
    type IntoIter = iterators::IntoGridLikeIterator<
        usize,
        T,
        std::iter::Map<
            std::iter::Enumerate<std::vec::IntoIter<T>>,
            impl FnMut((usize, T)) -> (grid_point::GridPoint<usize>, T),
        >,
    >;
    type Item = (grid_point::GridPoint<usize>, T);

    fn into_iter(self) -> Self::IntoIter {
        iterators::IntoGridLikeIterator::new(
            self.grid
                .into_iter()
                .enumerate()
                .map(move |(idx, value)| (point_of_arr_idx(self.dimensions, idx), value)),
        )
    }
}
