use std::error::Error;
use std::fmt::Display;
use std::ops::{Add, Mul, Neg};

/////////////
/// Grid Point
///
/// datastructure for indexing a grid
/////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPoint {
    row: usize,
    col: usize,
}

impl Display for GridPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl GridPoint {
    pub fn new(row: usize, col: usize) -> Self {
        GridPoint { row, col }
    }

    pub fn add(self, rhs: GridPointDelta, rows: usize, cols: usize) -> Option<Self> {
        let row: usize = ((self.row as isize) + rhs.row_delta).try_into().ok()?;
        let col: usize = ((self.col as isize) + rhs.col_delta).try_into().ok()?;

        if row < rows && col < cols {
            Some(GridPoint::new(row, col))
        } else {
            None
        }
    }

    fn as_arr_idx(&self, cols: usize) -> usize {
        self.row * cols + self.col
    }
}

////////////
/// Grid Point Delta
///
/// datastructure for adding to/subtracting from grid points
////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPointDelta {
    row_delta: isize,
    col_delta: isize,
}

impl Display for GridPointDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row_delta, self.row_delta)
    }
}

impl GridPointDelta {
    pub fn new(row_delta: isize, col_delta: isize) -> Self {
        GridPointDelta {
            row_delta,
            col_delta,
        }
    }

    pub fn zero() -> Self {
        GridPointDelta {
            row_delta: 0,
            col_delta: 0,
        }
    }
}

impl Neg for GridPointDelta {
    type Output = Self;

    fn neg(self) -> Self::Output {
        GridPointDelta {
            row_delta: -self.row_delta,
            col_delta: -self.col_delta,
        }
    }
}

impl Add<Self> for GridPointDelta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        GridPointDelta {
            row_delta: self.row_delta + rhs.row_delta,
            col_delta: self.col_delta + rhs.col_delta,
        }
    }
}

impl Mul<isize> for GridPointDelta {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        GridPointDelta {
            row_delta: self.row_delta * rhs,
            col_delta: self.col_delta * rhs,
        }
    }
}

pub const NORTH: GridPointDelta = GridPointDelta {
    row_delta: 0,
    col_delta: -1,
};
pub const EAST: GridPointDelta = GridPointDelta {
    row_delta: 1,
    col_delta: 0,
};
pub const SOUTH: GridPointDelta = GridPointDelta {
    row_delta: 0,
    col_delta: 1,
};
pub const WEST: GridPointDelta = GridPointDelta {
    row_delta: -1,
    col_delta: 0,
};

pub const NORTHEAST: GridPointDelta = GridPointDelta {
    row_delta: 1,
    col_delta: -1,
};
pub const SOUTHEAST: GridPointDelta = GridPointDelta {
    row_delta: 1,
    col_delta: 1,
};
pub const SOUTHWEST: GridPointDelta = GridPointDelta {
    row_delta: -1,
    col_delta: 1,
};
pub const NORTHWEST: GridPointDelta = GridPointDelta {
    row_delta: -1,
    col_delta: -1,
};

#[allow(dead_code)]
pub const PLUS_ADJACENT: [GridPointDelta; 4] = [NORTH, EAST, SOUTH, WEST];

#[allow(dead_code)]
pub const DIAG_ADJACENT: [GridPointDelta; 4] = [NORTHEAST, SOUTHEAST, SOUTHWEST, NORTHWEST];

#[allow(dead_code)]
pub const ADJACENT: [GridPointDelta; 8] = [
    NORTH, NORTHEAST, EAST, SOUTHEAST, SOUTH, SOUTHWEST, WEST, NORTHWEST,
];

////////////
/// Grid Point Iterators
///
/// helpful iterators for stepping over the grid
////////////

pub struct GridPointIterator {
    next: Option<GridPoint>,
    traverse_by: GridPointDelta,
    rows: usize,
    cols: usize,
}

impl Iterator for GridPointIterator {
    type Item = GridPoint;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next {
            self.next = next.add(self.traverse_by, self.rows, self.cols);
            Some(next)
        } else {
            None
        }
    }
}

impl GridPoint {
    pub fn traverse_by(
        self,
        traverse_by: GridPointDelta,
        rows: usize,
        cols: usize,
    ) -> GridPointIterator {
        GridPointIterator {
            next: Some(self),
            traverse_by,
            rows,
            cols,
        }
    }
}

////////////
/// Index Out Of Bounds error
///
/// error type for trying to index a part of the grid that doesn't exist
////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IndexOutOfBoundsError {
    rows: usize,
    cols: usize,
    attempted: GridPoint,
}

impl IndexOutOfBoundsError {
    #[inline]
    fn new(rows: usize, cols: usize, attempted: GridPoint) -> Self {
        IndexOutOfBoundsError {
            rows,
            cols,
            attempted,
        }
    }

    #[inline]
    fn err<T>(rows: usize, cols: usize, attempted: GridPoint) -> IndexResult<T> {
        Result::Err(IndexOutOfBoundsError::new(rows, cols, attempted))
    }
}

impl Display for IndexOutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attempted to access {} in grid with {} rows and {} cols",
            self.attempted, self.rows, self.cols,
        )
    }
}

impl Error for IndexOutOfBoundsError {}

type IndexResult<T> = Result<T, IndexOutOfBoundsError>;

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

    pub fn get(&self, point: GridPoint) -> IndexResult<&T> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn get_mut(&mut self, point: GridPoint) -> IndexResult<&mut T> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get_mut(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn set(&mut self, point: GridPoint, value: T) -> IndexResult<()> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        *self
            .grid
            .get_mut(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))? = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_init_test() {
        let grid_point: GridPoint = GridPoint::new(1, 1);
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
        let grid_point: GridPoint = GridPoint::new(1, 1);
        let step_by = SOUTH;

        assert_eq!(
            grid_point
                .traverse_by(step_by, 5, 5)
                .collect::<Vec<GridPoint>>(),
            vec![
                GridPoint { row: 1, col: 1 },
                GridPoint { row: 1, col: 2 },
                GridPoint { row: 1, col: 3 },
                GridPoint { row: 1, col: 4 }
            ]
        );
    }
}

/*
pub struct MarkedGrid<const ROWS: usize, const COLS: usize, T: Ord>
where
    [(); ROWS * COLS]:,
{
    grid: Grid<ROWS, COLS, T>,
    mask: Vector<{ ROWS * COLS }, bool>,
}

impl<const ROWS: usize, const COLS: usize, T: Ord> MarkedGrid<ROWS, COLS, T>
where
    [(); ROWS * COLS]:,
{
    pub fn new(grid: Grid<ROWS, COLS, T>) -> Self {
        MarkedGrid {
            grid,
            mask: Vector::constant(false),
        }
    }

    pub fn mark(&mut self, value: &T) {
        if let Some(indicies) = self.grid.index_lookup.get(value) {
            for idx in indicies {
                self.mask[*idx] = true;
            }
        }
    }
}

impl<const ROWS: usize, const COLS: usize, T: Ord> From<Grid<ROWS, COLS, T>>
    for MarkedGrid<ROWS, COLS, T>
where
    [(); ROWS * COLS]:,
{
    fn from(value: Grid<ROWS, COLS, T>) -> Self {
        MarkedGrid::new(value)
    }
}
*/
