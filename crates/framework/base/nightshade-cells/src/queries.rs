use crate::resources::{CellSize, Cells};
use nalgebra_glm::{IVec2, Vec2, vec2};
use nightshade_ecs::Storage;

pub fn walkable(cells: &Cells, storage: &Storage, cell: IVec2) -> bool {
    let Some(answer) = cells.walkable else {
        return true;
    };
    answer(storage, cell)
}

pub fn step_cost(cells: &Cells, storage: &Storage, cell: IVec2) -> f32 {
    let Some(answer) = cells.step_cost else {
        return 1.0;
    };
    answer(storage, cell)
}

pub fn in_bounds(cells: &Cells, storage: &Storage, cell: IVec2) -> bool {
    let Some(answer) = cells.in_bounds else {
        return true;
    };
    answer(storage, cell)
}

pub fn cell_of(size: &CellSize, spot: Vec2) -> IVec2 {
    IVec2::new(
        (spot.x / size.0).floor() as i32,
        (spot.y / size.0).floor() as i32,
    )
}

pub fn center_of(size: &CellSize, cell: IVec2) -> Vec2 {
    vec2(
        (cell.x as f32 + 0.5) * size.0,
        (cell.y as f32 + 0.5) * size.0,
    )
}

pub fn neighbours(cell: IVec2) -> [IVec2; 4] {
    [
        IVec2::new(cell.x + 1, cell.y),
        IVec2::new(cell.x - 1, cell.y),
        IVec2::new(cell.x, cell.y + 1),
        IVec2::new(cell.x, cell.y - 1),
    ]
}

pub fn manhattan(from: IVec2, to: IVec2) -> i32 {
    (from.x - to.x).abs() + (from.y - to.y).abs()
}
