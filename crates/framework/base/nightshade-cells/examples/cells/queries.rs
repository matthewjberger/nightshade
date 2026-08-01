use crate::theme;
use nalgebra_glm::IVec2;
use nightshade::ecs::Storage;
use nightshade_cells::queries::manhattan;

pub fn walkable_here(_storage: &Storage, cell: IVec2) -> bool {
    open(cell)
}

pub fn in_bounds_here(_storage: &Storage, cell: IVec2) -> bool {
    let half = theme::SIDE / 2;
    cell.x.abs() <= half && cell.y.abs() <= half
}

pub fn cost_here(_storage: &Storage, cell: IVec2) -> f32 {
    1.0 + manhattan(cell, IVec2::zeros()) as f32 * theme::COST_RISE
}

pub fn open(cell: IVec2) -> bool {
    let half = theme::SIDE / 2;
    if cell.x.abs() >= half || cell.y.abs() >= half {
        return false;
    }
    !(cell.x.rem_euclid(2) == 0 && cell.y.rem_euclid(2) == 0)
}
