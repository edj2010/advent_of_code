use std::collections::{hash_map, HashMap};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::iter::Step;
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Neg, Rem, Sub, SubAssign};

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

    pub fn contains(&self, point: &GridPoint<T>) -> bool
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
    pub fn of_points_inclusive(a: GridPoint<T>, b: GridPoint<T>) -> Self {
        GridDimensions {
            min_row: a.row.clone().min(b.row.clone()),
            max_row: Step::forward(a.row.max(b.row), 1),
            min_col: a.col.clone().min(b.col.clone()),
            max_col: Step::forward(a.col.max(b.col), 1),
        }
    }

    pub fn all_contained_points(self) -> GridDimensionIterator<T> {
        GridDimensionIterator {
            current_row: self.min_row.clone(),
            current_col: self.min_col.clone(),
            dimension: self,
        }
    }

    pub fn grow_to_contain(self, point: GridPoint<T>) -> Self {
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

/////////////
/// Grid Dimension Iterator
///
/// iterator over all gridpoints within a specified dimension
/////////////

pub struct GridDimensionIterator<T> {
    current_row: T,
    current_col: T,
    dimension: GridDimensions<T>,
}

impl<T: Step + Ord + Clone> Iterator for GridDimensionIterator<T> {
    type Item = GridPoint<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.dimension.max_row {
            return None;
        }
        let ret = GridPoint::new(self.current_row.clone(), self.current_col.clone());

        self.current_col = Step::forward(self.current_col.clone(), 1);
        if self.current_col >= self.dimension.max_col {
            self.current_col = self.dimension.min_col.clone();
            self.current_row = Step::forward(self.current_row.clone(), 1);
        }
        Some(ret)
    }
}

/////////////
/// Grid Point
///
/// datastructure for indexing a grid
/////////////
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPoint<T> {
    pub row: T,
    pub col: T,
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

impl<T, S> AddAssign<GridPointDelta<S>> for GridPoint<T>
where
    T: Clone,
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T>,
    <S as TryInto<T>>::Error: Debug,
    <S as TryFrom<T>>::Error: Debug,
{
    fn add_assign(&mut self, rhs: GridPointDelta<S>) {
        self.row = (S::try_from(self.row.clone()).unwrap() + rhs.row_delta)
            .try_into()
            .unwrap();
        self.col = (S::try_from(self.col.clone()).unwrap() + rhs.col_delta)
            .try_into()
            .unwrap();
    }
}

impl<T, S> SubAssign<GridPointDelta<S>> for GridPoint<T>
where
    T: Clone,
    S: TryInto<T> + Sub<S, Output = S> + TryFrom<T>,
    <S as TryInto<T>>::Error: Debug,
    <S as TryFrom<T>>::Error: Debug,
{
    fn sub_assign(&mut self, rhs: GridPointDelta<S>) {
        self.row = (S::try_from(self.row.clone()).unwrap() - rhs.row_delta)
            .try_into()
            .unwrap();
        self.col = (S::try_from(self.col.clone()).unwrap() - rhs.col_delta)
            .try_into()
            .unwrap();
    }
}

impl<T, S> Add<GridPointDelta<S>> for GridPoint<T>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T>,
    <S as TryInto<T>>::Error: Debug,
    <S as TryFrom<T>>::Error: Debug,
{
    type Output = GridPoint<T>;

    fn add(self, rhs: GridPointDelta<S>) -> Self::Output {
        let row: T = (S::try_from(self.row).unwrap() + rhs.row_delta)
            .try_into()
            .unwrap();
        let col: T = (S::try_from(self.col).unwrap() + rhs.col_delta)
            .try_into()
            .unwrap();

        GridPoint::new(row, col)
    }
}

impl<T, S> Sub<GridPointDelta<S>> for GridPoint<T>
where
    S: TryInto<T> + Sub<S, Output = S> + TryFrom<T>,
    <S as TryInto<T>>::Error: Debug,
    <S as TryFrom<T>>::Error: Debug,
{
    type Output = GridPoint<T>;

    fn sub(self, rhs: GridPointDelta<S>) -> Self::Output {
        let row: T = (S::try_from(self.row).unwrap() - rhs.row_delta)
            .try_into()
            .unwrap();
        let col: T = (S::try_from(self.col).unwrap() - rhs.col_delta)
            .try_into()
            .unwrap();

        GridPoint::new(row, col)
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

    pub fn as_type<U>(&self) -> GridPoint<U>
    where
        U: From<T>,
        T: Clone,
    {
        GridPoint {
            row: self.row.clone().into(),
            col: self.col.clone().into(),
        }
    }

    pub fn try_as_type<U>(&self) -> Result<GridPoint<U>, <U as TryFrom<T>>::Error>
    where
        U: TryFrom<T>,
        T: Clone,
    {
        Ok(GridPoint {
            row: self.row.clone().try_into()?,
            col: self.col.clone().try_into()?,
        })
    }

    pub fn add_checked<S>(
        self,
        rhs: GridPointDelta<S>,
        grid_dimensions: &GridDimensions<T>,
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

        let result = GridPoint::new(row, col);
        if grid_dimensions.contains(&result) {
            Some(result)
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

    pub fn as_type<U>(&self) -> GridPointDelta<U>
    where
        U: From<T>,
        T: Clone,
    {
        GridPointDelta {
            row_delta: self.row_delta.clone().into(),
            col_delta: self.col_delta.clone().into(),
        }
    }

    pub fn try_as_type<U>(&self) -> Result<GridPointDelta<U>, <U as TryFrom<T>>::Error>
    where
        U: TryFrom<T>,
        T: Clone,
    {
        Ok(GridPointDelta {
            row_delta: self.row_delta.clone().try_into()?,
            col_delta: self.col_delta.clone().try_into()?,
        })
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

impl GridPointDelta<i32> {
    pub fn l1_norm(self) -> i32 {
        self.row_delta.abs() + self.col_delta.abs()
    }
}

impl GridPointDelta<i64> {
    pub fn l1_norm(self) -> i64 {
        self.row_delta.abs() + self.col_delta.abs()
    }
}

fn gcd<T>(a: T, b: T) -> T
where
    T: Default + Ord + Neg<Output = T> + Rem<T, Output = T> + Copy,
{
    if a < T::default() {
        gcd(-a, b)
    } else if b < T::default() {
        gcd(a, -b)
    } else if b == T::default() {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl<T> Mul<T> for Direction
where
    GridPointDelta<T>: From<Direction>,
    T: Mul<T, Output = T> + Clone,
{
    type Output = GridPointDelta<T>;

    fn mul(self, rhs: T) -> Self::Output {
        GridPointDelta::<T>::from(self) * rhs
    }
}

impl Direction {
    pub fn all() -> [Self; 4] {
        [Self::North, Self::East, Self::South, Self::West]
    }

    pub fn rotate_left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    pub fn rotate_right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

impl From<Direction> for GridPointDelta<isize> {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => NORTH,
            Direction::East => EAST,
            Direction::South => SOUTH,
            Direction::West => WEST,
        }
    }
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

pub const ZERO: GridPointDelta<isize> = GridPointDelta {
    row_delta: 0,
    col_delta: 0,
};
pub const NORTH: GridPointDelta<isize> = GridPointDelta {
    row_delta: -1,
    col_delta: 0,
};
pub const EAST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 0,
    col_delta: 1,
};
pub const SOUTH: GridPointDelta<isize> = GridPointDelta {
    row_delta: 1,
    col_delta: 0,
};
pub const WEST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 0,
    col_delta: -1,
};

pub const NORTHEAST: GridPointDelta<isize> = GridPointDelta {
    row_delta: -1,
    col_delta: 1,
};
pub const SOUTHEAST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 1,
    col_delta: 1,
};
pub const SOUTHWEST: GridPointDelta<isize> = GridPointDelta {
    row_delta: 1,
    col_delta: -1,
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
    grid_dimensions: GridDimensions<T>,
}

impl<T, S> Iterator for GridPointIterator<T, S>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    T: PartialOrd + Clone,
{
    type Item = GridPoint<T>;

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

impl<T> GridPoint<T> {
    pub fn traverse_by<S>(
        self,
        traverse_by: GridPointDelta<S>,
        grid_dimensions: GridDimensions<T>,
    ) -> GridPointIterator<T, S> {
        GridPointIterator {
            next: Some(self),
            traverse_by,
            grid_dimensions,
        }
    }
}

impl GridPoint<usize> {
    pub fn traverse_to(self, other: Self) -> Option<GridPointIterator<usize, isize>> {
        Some(GridPointIterator {
            next: Some(self),
            traverse_by: other.sub(self)?.min_step(),
            grid_dimensions: GridDimensions::<usize>::of_points_inclusive(self, other),
        })
    }
}

impl GridPoint<isize> {
    pub fn traverse_to(self, other: Self) -> Option<GridPointIterator<isize, isize>> {
        Some(GridPointIterator {
            next: Some(self),
            traverse_by: other.sub(self)?.min_step(),
            grid_dimensions: GridDimensions::<isize>::of_points_inclusive(self, other),
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

#[derive(Clone, PartialEq, Eq)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    grid: Vec<T>,
}

impl<T: Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                write!(f, "{:?} ", self.get(GridPoint::new(row, col)))?
            }
            write!(f, "\n")?
        }
        write!(f, "")
    }
}

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                write!(f, "{} ", self.get(GridPoint::new(row, col)).unwrap())?
            }
            write!(f, "\n")?
        }
        write!(f, "")
    }
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
        let mut grid: Vec<T> = Vec::with_capacity(rows * cols);
        for (row_idx, row) in v.into_iter().enumerate() {
            grid.extend(row);
            if grid.len() != cols * (row_idx + 1) {
                return None;
            }
        }
        Some(Grid { rows, cols, grid })
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

    pub fn dimensions(&self) -> GridDimensions<usize> {
        GridDimensions::new(0, self.rows, 0, self.cols)
    }

    pub fn iter_points(&self) -> GridDimensionIterator<usize> {
        let dimensions = self.dimensions();
        dimensions.all_contained_points()
    }

    pub fn get(&self, point: GridPoint<usize>) -> IndexResult<&T, usize> {
        if point.row >= self.rows || point.col >= self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn get_mut(&mut self, point: GridPoint<usize>) -> IndexResult<&mut T, usize> {
        if point.row >= self.rows || point.col >= self.cols {
            return IndexOutOfBoundsError::err(self.rows, self.cols, point);
        }
        self.grid
            .get_mut(point.as_arr_idx(self.cols))
            .ok_or(IndexOutOfBoundsError::new(self.rows, self.cols, point))
    }

    pub fn set(&mut self, point: GridPoint<usize>, value: T) -> IndexResult<(), usize> {
        if point.row >= self.rows || point.col >= self.cols {
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

    pub fn all_rows(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.grid.chunks(self.cols).map(|s| s.into()).collect()
    }

    pub fn all_cols(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        GridPoint::new(0, 0)
            .traverse_by(EAST, self.dimensions())
            .map(|col_start| {
                col_start
                    .traverse_by(SOUTH, self.dimensions())
                    .map(|point| self.get(point).unwrap().clone())
                    .collect()
            })
            .collect()
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

    pub fn from<I: IntoIterator<Item = (GridPoint<isize>, T)>>(iter: I) -> Self {
        let mut lattice = Self::empty();
        for (point, value) in iter.into_iter() {
            lattice.set(point, value);
        }
        lattice
    }
}

impl<T> Lattice<T> {
    pub fn contains(&self, point: GridPoint<isize>) -> bool {
        self.get(point).is_some()
    }

    pub fn get(&self, point: GridPoint<isize>) -> Option<&T> {
        self.points.get(&point)
    }

    pub fn get_mut(&mut self, point: GridPoint<isize>) -> Option<&mut T> {
        self.points.get_mut(&point)
    }

    pub fn set(&mut self, point: GridPoint<isize>, value: T) -> Option<T> {
        self.points.insert(point, value)
    }

    pub fn entry(&mut self, point: GridPoint<isize>) -> LatticeEntry<'_, T> {
        LatticeEntry(self.points.entry(point))
    }

    pub fn bounding_box(&self) -> Option<GridDimensions<isize>> {
        self.points
            .iter()
            .fold(None, |grid_dimension, (&point, _)| match grid_dimension {
                None => Some(GridDimensions::of_points_inclusive(point, point)),
                Some(dimension) => Some(dimension.grow_to_contain(point)),
            })
    }

    pub fn intersects_block(&self, block: &Block<isize>) -> bool {
        block.0.iter().any(|p| self.contains(*p))
    }

    pub fn set_block(&mut self, block: Block<isize>, value: T)
    where
        T: Clone,
    {
        block.0.into_iter().for_each(|p| {
            self.set(p, value.clone());
        })
    }

    pub fn apply_block_with_default<F: Fn(&mut T, GridPoint<isize>)>(
        &mut self,
        block: Block<isize>,
        f: F,
    ) where
        T: Clone + Default,
    {
        block
            .0
            .into_iter()
            .for_each(|p| f(self.entry(p).or_default(), p))
    }

    pub fn apply_block<F: Fn(&mut T, GridPoint<isize>)>(
        &mut self,
        block: Block<isize>,
        f: F,
        default: T,
    ) where
        T: Clone + Default,
    {
        block
            .0
            .into_iter()
            .for_each(|p| f(self.entry(p).or_insert(default.clone()), p))
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
/// LatticeEntry
///
/// Used for modifying lattice entries
////////////

pub struct LatticeEntry<'a, T: 'a>(hash_map::Entry<'a, GridPoint<isize>, T>);

impl<'a, T> LatticeEntry<'a, T> {
    pub fn or_insert(self, default: T) -> &'a mut T {
        self.0.or_insert(default)
    }

    pub fn or_insert_with<F: FnOnce() -> T>(self, default: F) -> &'a mut T {
        self.0.or_insert_with(default)
    }

    pub fn or_insert_with_key<F: FnOnce(&GridPoint<isize>) -> T>(self, default: F) -> &'a mut T {
        self.0.or_insert_with_key(default)
    }

    pub fn key(&self) -> &GridPoint<isize> {
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

////////////
/// LatticeIterator + LatticeIteratorMut + IntoLatticeIterator
///
/// iterates through all non-empty lattice points
////////////

pub struct LatticeIterator<'a, T>(hash_map::Iter<'a, GridPoint<isize>, T>);

impl<'a, T> Iterator for LatticeIterator<'a, T> {
    type Item = (&'a GridPoint<isize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct LatticeIteratorMut<'a, T>(hash_map::IterMut<'a, GridPoint<isize>, T>);

impl<'a, T> Iterator for LatticeIteratorMut<'a, T> {
    type Item = (&'a GridPoint<isize>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct IntoLatticeIterator<T>(hash_map::IntoIter<GridPoint<isize>, T>);

impl<T> Iterator for IntoLatticeIterator<T> {
    type Item = (GridPoint<isize>, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> IntoIterator for Lattice<T> {
    type IntoIter = IntoLatticeIterator<T>;
    type Item = (GridPoint<isize>, T);

    fn into_iter(self) -> Self::IntoIter {
        IntoLatticeIterator(self.points.into_iter())
    }
}

////////////
/// Block
///
/// very small grid, designed to be moved around,
/// checked for intersection, and added to a lattice or grid
////////////

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn dimensions(&self) -> Option<GridDimensions<&T>>
    where
        T: Ord,
    {
        Some(GridDimensions {
            min_row: self.min_row()?,
            max_row: self.max_row()?,
            min_col: self.min_col()?,
            max_col: self.max_col()?,
        })
    }

    pub fn add_checked<S>(
        self,
        rhs: GridPointDelta<S>,
        grid_dimensions: &GridDimensions<T>,
    ) -> Option<Block<T>>
    where
        T: PartialOrd,
        S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    {
        Some(Block(
            self.0
                .into_iter()
                .map(|p| p.add_checked(rhs.clone(), grid_dimensions))
                .collect::<Option<Vec<GridPoint<T>>>>()?,
        ))
    }
}

impl<T, S> Add<GridPointDelta<S>> for Block<T>
where
    S: TryInto<T> + Add<S, Output = S> + TryFrom<T> + Clone,
    <S as TryInto<T>>::Error: Debug,
    <S as TryFrom<T>>::Error: Debug,
{
    type Output = Block<T>;

    fn add(self, rhs: GridPointDelta<S>) -> Self::Output {
        Block(
            self.0
                .into_iter()
                .map(|p| p + rhs.clone())
                .collect::<Vec<GridPoint<T>>>(),
        )
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
                .traverse_by(step_by, GridDimensions::new(0, 5, 0, 5))
                .collect::<Vec<GridPoint<usize>>>(),
            vec![
                GridPoint { row: 1, col: 1 },
                GridPoint { row: 2, col: 1 },
                GridPoint { row: 3, col: 1 },
                GridPoint { row: 4, col: 1 }
            ]
        );
    }
}
