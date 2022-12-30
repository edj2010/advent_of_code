use std::cmp::{max, min};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Rem, Sub};

/////////////
/// Grid Point
///
/// datastructure for indexing a grid
/////////////
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPoint<T> {
    row: T,
    col: T,
}

impl<T> Debug for GridPoint<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.row, self.col)
    }
}

impl<T> Display for GridPoint<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl<T, S> Add<GridPointDelta<S>> for GridPoint<T>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T>,
{
    type Output = Option<GridPoint<T>>;

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

impl<T> GridPoint<T> {
    pub fn new(row: T, col: T) -> Self {
        GridPoint { row, col }
    }

    pub fn row(&self) -> &T {
        &self.row
    }

    pub fn col(&self) -> &T {
        &self.col
    }

    pub fn sub<S>(self, rhs: Self) -> Option<GridPointDelta<S>>
    where
        S: TryInto<T> + Sub<S, Output = S> + TryFrom<T>,
    {
        let row_delta: S = S::try_from(self.row).ok()? - S::try_from(rhs.row).ok()?;
        let col_delta: S = S::try_from(self.col).ok()? - S::try_from(rhs.col).ok()?;
        Some(GridPointDelta {
            row_delta,
            col_delta,
        })
    }

    pub fn add_checked<S>(
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

    pub fn row_delta(&self) -> &T {
        &self.row_delta
    }

    pub fn col_delta(&self) -> &T {
        &self.col_delta
    }
}

impl GridPointDelta<isize> {
    pub fn l1_norm(self) -> isize {
        self.row_delta.abs() + self.col_delta.abs()
    }
}

fn gcd(a: isize, b: isize) -> isize {
    if a < 0 || b < 0 {
        gcd(a.abs(), b.abs())
    } else if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

impl GridPointDelta<isize> {
    pub fn zero() -> Self {
        GridPointDelta {
            row_delta: 0,
            col_delta: 0,
        }
    }

    pub fn min_step(self) -> Self {
        let div = gcd(self.row_delta, self.col_delta);
        GridPointDelta {
            row_delta: self.row_delta / div,
            col_delta: self.col_delta / div,
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
    next: Option<GridPoint<T>>,
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
    type Item = GridPoint<T>;

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

impl<T> GridPoint<T> {
    pub fn traverse_by<S>(
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

impl GridPoint<usize> {
    pub fn traverse_to(self, other: Self) -> Option<GridPointIterator<usize, isize>> {
        Some(GridPointIterator {
            next: Some(self),
            traverse_by: other.sub(self)?.min_step(),
            min_row: min(self.row, other.row),
            max_row: max(self.row, other.row) + 1,
            min_col: min(self.col, other.col),
            max_col: max(self.col, other.col) + 1,
        })
    }
}

impl GridPoint<isize> {
    pub fn traverse_to(self, other: Self) -> Option<GridPointIterator<isize, isize>> {
        Some(GridPointIterator {
            next: Some(self),
            traverse_by: other.sub(self)?.min_step(),
            min_row: min(self.row, other.row),
            max_row: max(self.row, other.row) + 1,
            min_col: min(self.col, other.col),
            max_col: max(self.col, other.col) + 1,
        })
    }
}

////////////
/// Index Out Of Bounds error
///
/// error type for trying to index a part of the grid that doesn't exist
////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IndexOutOfBoundsError<T> {
    rows: usize,
    cols: usize,
    attempted: GridPoint<T>,
}

impl<T> IndexOutOfBoundsError<T> {
    #[inline]
    fn new(rows: usize, cols: usize, attempted: GridPoint<T>) -> Self {
        IndexOutOfBoundsError {
            rows,
            cols,
            attempted,
        }
    }

    #[inline]
    fn err<U>(rows: usize, cols: usize, attempted: GridPoint<T>) -> IndexResult<U, T> {
        Result::Err(IndexOutOfBoundsError::new(rows, cols, attempted))
    }
}

impl<T: Display> Display for IndexOutOfBoundsError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attempted to access {} in grid with {} rows and {} cols",
            self.attempted, self.rows, self.cols,
        )
    }
}

impl<T: Debug + Display> Error for IndexOutOfBoundsError<T> {}

type IndexResult<T, S> = Result<T, IndexOutOfBoundsError<S>>;

////////////
/// Grid
///
/// simple datastructure for holding a grid of values
////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    grid: Vec<T>,
}

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

    pub fn get(&self, point: GridPoint<usize>) -> IndexResult<&T, usize> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn get_mut(&mut self, point: GridPoint<usize>) -> IndexResult<&mut T, usize> {
        if point.row > self.rows || point.col > self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get_mut(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn set(&mut self, point: GridPoint<usize>, value: T) -> IndexResult<(), usize> {
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

impl<T> Index<GridPoint<usize>> for Grid<T> {
    type Output = T;

    fn index(&self, index: GridPoint<usize>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<GridPoint<usize>> for Grid<T> {
    fn index_mut(&mut self, index: GridPoint<usize>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: PartialEq> Grid<T> {
    pub fn find(&self, val: &T) -> Option<GridPoint<usize>> {
        self.grid
            .iter()
            .enumerate()
            .find(|(_, v)| &val == v)
            .map(|(idx, _)| GridPoint::of_arr_idx(idx, self.cols))
    }
}

////////////
/// Lattice
///
/// similar interface to grid but allows for more efficient storage of
/// sparse data and allows for negative indicies
////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lattice<T> {
    points: HashMap<GridPoint<isize>, T>,
}

impl<T: Clone> Lattice<T> {
    pub fn empty() -> Self {
        Lattice {
            points: HashMap::new(),
        }
    }
}

impl<T> Lattice<T> {
    pub fn get(&self, point: GridPoint<isize>) -> Option<&T> {
        self.points.get(&point)
    }

    pub fn get_mut(&mut self, point: GridPoint<isize>) -> Option<&mut T> {
        self.points.get_mut(&point)
    }

    pub fn set(&mut self, point: GridPoint<isize>, value: T) -> Option<T> {
        self.points.insert(point, value)
    }
}

impl<T> Index<GridPoint<isize>> for Lattice<T> {
    type Output = T;

    fn index(&self, index: GridPoint<isize>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<GridPoint<isize>> for Lattice<T> {
    fn index_mut(&mut self, index: GridPoint<isize>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: PartialEq> Lattice<T> {
    pub fn find(&self, val: &T) -> Option<GridPoint<isize>> {
        self.points
            .iter()
            .find(|(_, v)| &val == v)
            .map(|(idx, _)| idx)
            .copied()
    }
}

////////////
/// Block
///
/// very small grid, designed to be moved around,
/// checked for intersection, and added to a lattice or grid
////////////

pub struct Block<T>(Vec<GridPoint<T>>);

impl<T> Block<T> {
    pub fn empty() -> Self {
        Block(Vec::new())
    }

    pub fn from<I: IntoIterator<Item = GridPoint<T>>>(iter: I) -> Self {
        Block(iter.into_iter().collect())
    }

    pub fn add_point(&mut self, point: GridPoint<T>) {
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
}

impl<T, S> Add<GridPointDelta<S>> for Block<T>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
{
    type Output = Option<Block<T>>;

    fn add(self, rhs: GridPointDelta<S>) -> Self::Output {
        Some(Block(
            self.0
                .into_iter()
                .map(|p| p + rhs.clone())
                .collect::<Option<Vec<GridPoint<T>>>>()?,
        ))
    }
}

////////////
/// Tests
////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_init_test() {
        let grid_point: GridPoint<usize> = GridPoint::new(1, 1);
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
        let grid_point: GridPoint<usize> = GridPoint::new(1, 1);
        let step_by = SOUTH;

        assert_eq!(
            grid_point
                .traverse_by(step_by, 0, 5, 0, 5)
                .collect::<Vec<GridPoint<usize>>>(),
            vec![
                GridPoint { row: 1, col: 1 },
                GridPoint { row: 1, col: 2 },
                GridPoint { row: 1, col: 3 },
                GridPoint { row: 1, col: 4 }
            ]
        );
    }
}
