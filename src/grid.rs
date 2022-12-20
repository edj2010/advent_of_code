use std::error::Error;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Rem, Sub};

/////////////
/// Grid Point
///
/// datastructure for indexing a grid
/////////////
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPoint<T, S> {
    row: T,
    col: T,
    _delta_type: PhantomData<S>,
}

impl<T, S> Debug for GridPoint<T, S>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.row, self.col)
    }
}

impl<T, S> Display for GridPoint<T, S>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl<T, S> Add<GridPointDelta<S>> for GridPoint<T, S>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T>,
{
    type Output = Option<GridPoint<T, S>>;

    fn add(self, rhs: GridPointDelta<S>) -> Self::Output {
        let row: T = (S::try_from(self.row).ok()? + rhs.row_delta)
            .try_into()
            .ok()?;
        let col: T = (S::try_from(self.col).ok()? + rhs.col_delta)
            .try_into()
            .ok()?;

        Some(GridPoint::new(row, col))
    }
}

impl<T, S> Sub<GridPoint<T, S>> for GridPoint<T, S>
where
    T: TryInto<S>,
    S: Sub<S, Output = S> + TryFrom<T>,
{
    type Output = Option<GridPointDelta<S>>;

    fn sub(self, rhs: GridPoint<T, S>) -> Self::Output {
        let row_delta: S = S::try_from(self.row).ok()? - S::try_from(rhs.row).ok()?;
        let col_delta: S = S::try_from(self.col).ok()? - S::try_from(rhs.col).ok()?;
        Some(GridPointDelta {
            row_delta,
            col_delta,
        })
    }
}

impl<T, S> GridPoint<T, S> {
    pub fn new(row: T, col: T) -> Self {
        GridPoint {
            row,
            col,
            _delta_type: PhantomData,
        }
    }

    pub fn add_checked(
        self,
        rhs: GridPointDelta<S>,
        min_row: &T,
        max_row: &T,
        min_col: &T,
        max_col: &T,
    ) -> Option<Self>
    where
        S: TryInto<T> + Add<S, Output = S> + TryFrom<T>,
        T: PartialOrd,
    {
        let row: T = (S::try_from(self.row).ok()? + rhs.row_delta)
            .try_into()
            .ok()?;
        let col: T = (S::try_from(self.col).ok()? + rhs.col_delta)
            .try_into()
            .ok()?;

        if min_row <= &row && &row < max_row && min_col <= &col && &col < max_col {
            Some(GridPoint::new(row, col))
        } else {
            None
        }
    }

    fn as_arr_idx(&self, cols: T) -> T
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Clone,
    {
        self.row.clone() * cols + self.col.clone()
    }

    fn of_arr_idx(arr_idx: T, cols: T) -> Self
    where
        T: Div<T, Output = T> + Rem<T, Output = T> + Clone,
    {
        GridPoint {
            row: arr_idx.clone() / cols.clone(),
            col: arr_idx % cols,
            _delta_type: PhantomData,
        }
    }
}

////////////
/// Grid Point Delta
///
/// datastructure for adding to/subtracting from grid points
////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPointDelta<T> {
    pub row_delta: T,
    pub col_delta: T,
}

impl<T> Display for GridPointDelta<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row_delta, self.col_delta)
    }
}

impl<T> GridPointDelta<T> {
    pub fn new(row_delta: T, col_delta: T) -> Self {
        GridPointDelta {
            row_delta,
            col_delta,
        }
    }
}

impl GridPointDelta<isize> {
    pub fn zero() -> Self {
        GridPointDelta {
            row_delta: 0,
            col_delta: 0,
        }
    }
}

impl<T> Neg for GridPointDelta<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        GridPointDelta {
            row_delta: -self.row_delta,
            col_delta: -self.col_delta,
        }
    }
}

impl<T> Add<Self> for GridPointDelta<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        GridPointDelta {
            row_delta: self.row_delta + rhs.row_delta,
            col_delta: self.col_delta + rhs.col_delta,
        }
    }
}

impl<T> Mul<T> for GridPointDelta<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        GridPointDelta {
            row_delta: self.row_delta * rhs.clone(),
            col_delta: self.col_delta * rhs,
        }
    }
}

pub const ZERO: GridPointDelta<isize> = GridPointDelta {
    row_delta: 0,
    col_delta: 0,
};
pub const NORTH: GridPointDelta<isize> = GridPointDelta {
    row_delta: 0,
    col_delta: -1,
};
pub const EAST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 1,
    col_delta: 0,
};
pub const SOUTH: GridPointDelta<isize> = GridPointDelta {
    row_delta: 0,
    col_delta: 1,
};
pub const WEST: GridPointDelta<isize> = GridPointDelta {
    row_delta: -1,
    col_delta: 0,
};

pub const NORTHEAST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 1,
    col_delta: -1,
};
pub const SOUTHEAST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 1,
    col_delta: 1,
};
pub const SOUTHWEST: GridPointDelta<isize> = GridPointDelta {
    row_delta: -1,
    col_delta: 1,
};
pub const NORTHWEST: GridPointDelta<isize> = GridPointDelta {
    row_delta: -1,
    col_delta: -1,
};

#[allow(dead_code)]
pub const PLUS_ADJACENT: [GridPointDelta<isize>; 4] = [NORTH, EAST, SOUTH, WEST];

#[allow(dead_code)]
pub const DIAG_ADJACENT: [GridPointDelta<isize>; 4] = [NORTHEAST, SOUTHEAST, SOUTHWEST, NORTHWEST];

#[allow(dead_code)]
pub const ADJACENT: [GridPointDelta<isize>; 8] = [
    NORTH, NORTHEAST, EAST, SOUTHEAST, SOUTH, SOUTHWEST, WEST, NORTHWEST,
];

////////////
/// Grid Point Iterators
///
/// helpful iterators for stepping over the grid
////////////

pub struct GridPointIterator<T, S> {
    next: Option<GridPoint<T, S>>,
    traverse_by: GridPointDelta<S>,
    min_row: T,
    max_row: T,
    min_col: T,
    max_col: T,
}

impl<T, S> Iterator for GridPointIterator<T, S>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    T: PartialOrd + Clone,
{
    type Item = GridPoint<T, S>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.clone() {
            self.next = next.clone().add_checked(
                self.traverse_by.clone(),
                &self.min_row,
                &self.max_row,
                &self.min_col,
                &self.max_col,
            );
            Some(next)
        } else {
            None
        }
    }
}

impl<T, S> GridPoint<T, S> {
    pub fn traverse_by(
        self,
        traverse_by: GridPointDelta<S>,
        min_row: T,
        max_row: T,
        min_col: T,
        max_col: T,
    ) -> GridPointIterator<T, S> {
        GridPointIterator {
            next: Some(self),
            traverse_by,
            min_row,
            max_row,
            min_col,
            max_col,
        }
    }
}

////////////
/// Index Out Of Bounds error
///
/// error type for trying to index a part of the grid that doesn't exist
////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IndexOutOfBoundsError<T, S> {
    rows: usize,
    cols: usize,
    attempted: GridPoint<T, S>,
}

impl<T, S> IndexOutOfBoundsError<T, S> {
    #[inline]
    fn new(rows: usize, cols: usize, attempted: GridPoint<T, S>) -> Self {
        IndexOutOfBoundsError {
            rows,
            cols,
            attempted,
        }
    }

    #[inline]
    fn err<U>(rows: usize, cols: usize, attempted: GridPoint<T, S>) -> IndexResult<U, T, S> {
        Result::Err(IndexOutOfBoundsError::new(rows, cols, attempted))
    }
}

impl<T: Display, S> Display for IndexOutOfBoundsError<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attempted to access {} in grid with {} rows and {} cols",
            self.attempted, self.rows, self.cols,
        )
    }
}

impl<T: Debug + Display, S: Debug + Display> Error for IndexOutOfBoundsError<T, S> {}

type IndexResult<T, S, U> = Result<T, IndexOutOfBoundsError<S, U>>;

////////////
/// Grid
///
/// simple datastructure for holding a grid of values
////////////

pub struct Grid<T> {
    rows: usize,
    cols: usize,
    grid: Vec<T>,
}

impl<T> Debug for Grid<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:?}", self.rows, self.cols, self.grid)
    }
}

impl<T: Clone> Clone for Grid<T> {
    #[inline]
    fn clone(&self) -> Self {
        Grid {
            rows: self.rows,
            cols: self.cols,
            grid: self.grid.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for Grid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols && self.grid == other.grid
    }
}

impl<T: Eq> Eq for Grid<T> {}

impl<T: Clone> Grid<T> {
    pub fn init(init: T, rows: usize, cols: usize) -> Self {
        Grid {
            rows,
            cols,
            grid: vec![init; rows * cols],
        }
    }

    pub fn from<I: IntoIterator<Item = T>>(v: I, rows: usize, cols: usize) -> Option<Self> {
        let grid: Vec<T> = v.into_iter().collect::<Vec<T>>();
        if grid.len() == rows * cols {
            Some(Grid { rows, cols, grid })
        } else {
            None
        }
    }

    pub fn of_list_of_lists<J: IntoIterator<Item = T>, I: IntoIterator<Item = J>>(
        v: I,
        rows: usize,
        cols: usize,
    ) -> Option<Self> {
        Self::from(v.into_iter().flat_map(|i| i.into_iter()), rows, cols)
    }

    pub fn of_vec_of_vecs(v: Vec<Vec<T>>) -> Option<Self> {
        let rows = v.len();
        let cols = v[0].len();
        Self::of_list_of_lists(v, rows, cols)
    }
}

impl<T> Grid<T> {
    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn get(&self, point: GridPoint<usize, isize>) -> IndexResult<&T, usize, isize> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn get_mut(&mut self, point: GridPoint<usize, isize>) -> IndexResult<&mut T, usize, isize> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get_mut(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn set(
        &mut self,
        point: GridPoint<usize, isize>,
        value: T,
    ) -> IndexResult<(), usize, isize> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        *self
            .grid
            .get_mut(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))? = value;
        Ok(())
    }

    pub fn into_iter(self) -> std::vec::IntoIter<T> {
        self.grid.into_iter()
    }
}

impl<T> Index<GridPoint<usize, isize>> for Grid<T> {
    type Output = T;

    fn index(&self, index: GridPoint<usize, isize>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<GridPoint<usize, isize>> for Grid<T> {
    fn index_mut(&mut self, index: GridPoint<usize, isize>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: PartialEq> Grid<T> {
    pub fn find(&self, val: &T) -> Option<GridPoint<usize, isize>> {
        self.grid
            .iter()
            .enumerate()
            .find(|(_, v)| &val == v)
            .map(|(idx, _)| GridPoint::of_arr_idx(idx, self.cols))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_init_test() {
        let grid_point: GridPoint<usize, isize> = GridPoint::new(1, 1);
        let grid: Grid<char> = Grid::from(
            "123
456
789"
            .lines()
            .map(|line| line.chars())
            .flatten(),
            3,
            3,
        )
        .unwrap();
        assert_eq!(grid.get(grid_point), Ok(&'5'));
    }

    #[test]
    fn grid_iter_test() {
        let grid_point: GridPoint<usize, isize> = GridPoint::new(1, 1);
        let step_by = SOUTH;

        assert_eq!(
            grid_point
                .traverse_by(step_by, 0, 5, 0, 5)
                .collect::<Vec<GridPoint<usize, isize>>>(),
            vec![
                GridPoint {
                    row: 1,
                    col: 1,
                    _delta_type: PhantomData
                },
                GridPoint {
                    row: 1,
                    col: 2,
                    _delta_type: PhantomData
                },
                GridPoint {
                    row: 1,
                    col: 3,
                    _delta_type: PhantomData
                },
                GridPoint {
                    row: 1,
                    col: 4,
                    _delta_type: PhantomData
                }
            ]
        );
    }
}
