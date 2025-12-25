use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul, Neg, Rem, Sub, SubAssign},
};

use super::grid_dimension;
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
        grid_dimensions: &grid_dimension::GridDimensions<T>,
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

macro_rules! grid_point_delta_impl {
    ($integer:ty) => {
        impl GridPointDelta<$integer> {
            pub fn l1_norm(self) -> $integer {
                self.row_delta.abs() + self.col_delta.abs()
            }

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
    };
}

grid_point_delta_impl!(i8);
grid_point_delta_impl!(i16);
grid_point_delta_impl!(i32);
grid_point_delta_impl!(i64);
grid_point_delta_impl!(i128);
grid_point_delta_impl!(isize);
