use super::{constants, grid_point};
use std::ops::{Mul, Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl<T> Mul<T> for Direction
where
    grid_point::GridPointDelta<T>: From<Direction>,
    T: Mul<T, Output = T> + Clone,
{
    type Output = grid_point::GridPointDelta<T>;

    fn mul(self, rhs: T) -> Self::Output {
        grid_point::GridPointDelta::<T>::from(self) * rhs
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

impl From<Direction> for grid_point::GridPointDelta<isize> {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => constants::NORTH,
            Direction::East => constants::EAST,
            Direction::South => constants::SOUTH,
            Direction::West => constants::WEST,
        }
    }
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::North => Direction::South,
            Self::East => Direction::West,
            Self::South => Direction::North,
            Self::West => Direction::East,
        }
    }
}
