mod block;
mod constants;
mod direction;
mod error;
mod grid;
mod grid_dimension;
mod grid_point;
mod iterators;
mod lattice;

pub use block::Block;
pub use constants::*;
pub use direction::Direction;
pub use grid::Grid;
pub use grid_dimension::GridDimensions;
pub use grid_point::{GridPoint, GridPointDelta};
pub use lattice::Lattice;

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
