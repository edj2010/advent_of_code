use super::grid_point;

pub const ZERO: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: 0,
    col_delta: 0,
};
pub const NORTH: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: -1,
    col_delta: 0,
};
pub const EAST: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: 0,
    col_delta: 1,
};
pub const SOUTH: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: 1,
    col_delta: 0,
};
pub const WEST: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: 0,
    col_delta: -1,
};

pub const NORTHEAST: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: -1,
    col_delta: 1,
};
pub const SOUTHEAST: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: 1,
    col_delta: 1,
};
pub const SOUTHWEST: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: 1,
    col_delta: -1,
};
pub const NORTHWEST: grid_point::GridPointDelta<isize> = grid_point::GridPointDelta {
    row_delta: -1,
    col_delta: -1,
};

#[allow(dead_code)]
pub const PLUS_ADJACENT: [grid_point::GridPointDelta<isize>; 4] = [NORTH, EAST, SOUTH, WEST];

#[allow(dead_code)]
pub const DIAG_ADJACENT: [grid_point::GridPointDelta<isize>; 4] =
    [NORTHEAST, SOUTHEAST, SOUTHWEST, NORTHWEST];

#[allow(dead_code)]
pub const ADJACENT: [grid_point::GridPointDelta<isize>; 8] = [
    NORTH, NORTHEAST, EAST, SOUTHEAST, SOUTH, SOUTHWEST, WEST, NORTHWEST,
];
