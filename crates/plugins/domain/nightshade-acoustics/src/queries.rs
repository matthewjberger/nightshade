use crate::contract::{AcousticsGrid, AcousticsHeard, SoundPath};
use nightshade_ecs::Entity;

pub fn acoustics_path_of(heard: &AcousticsHeard, entity: Entity) -> Option<SoundPath> {
    heard
        .paths
        .iter()
        .find(|(found, _)| *found == entity)
        .map(|(_, path)| *path)
}

pub fn acoustics_grid_index(grid: &AcousticsGrid, cell: [u32; 3]) -> Option<usize> {
    if cell[0] >= grid.side || cell[1] >= grid.side || cell[2] >= grid.side {
        return None;
    }
    let side = grid.side as usize;
    Some((cell[2] as usize * side + cell[1] as usize) * side + cell[0] as usize)
}
