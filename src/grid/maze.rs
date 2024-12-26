use super::super::search;
use super::{direction, grid, grid_point};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MazeCell {
    Empty,
    Wall,
}

pub struct Maze<
    F: Fn(
        (grid_point::GridPoint<usize>, direction::Direction),
        (grid_point::GridPoint<usize>, direction::Direction),
    ) -> u64,
> {
    grid: grid::Grid<MazeCell>,
    move_cost: F,
    end: Option<grid_point::GridPoint<usize>>,
}

impl<
        F: Fn(
            (grid_point::GridPoint<usize>, direction::Direction),
            (grid_point::GridPoint<usize>, direction::Direction),
        ) -> u64,
    > search::WeightedGraphWithHeuristic for Maze<F>
{
    type Key = (grid_point::GridPoint<usize>, direction::Direction);
    type Cost = u64;
    type Weight = u64;

    fn adjacent(&self, k: &Self::Key) -> Option<impl Iterator<Item = Self::Key>> {
        let (current, _) = k;
        Some(
            direction::Direction::all()
                .into_iter()
                .filter_map(|direction| {
                    Some((
                        current.add_checked(direction.into(), &self.grid.dimensions())?,
                        direction,
                    ))
                })
                .filter(|(point, _)| self.grid.get(*point) != Ok(&MazeCell::Wall)),
        )
    }

    fn cost(&self, a: &Self::Key, b: &Self::Key) -> Option<Self::Cost> {
        Some((self.move_cost)(*a, *b))
    }

    fn cost_to_weight(&self, k: &Self::Key, c: Self::Cost) -> Self::Weight {
        if let Some(end) = self.end {
            (end.sub::<i64>(k.0)).unwrap().l1_norm() as u64 + c
        } else {
            c
        }
    }
}
