use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::queries::{block_at, is_solid_at};
use blocker_blocks::resources::VoxelWorld;
use nalgebra_glm::{IVec2, IVec3};

pub fn cell_corner(anchor: IVec3, index: usize) -> IVec2 {
    let across = theme::CELLS_ACROSS as i32;
    let home = IVec2::new(
        anchor.x.div_euclid(theme::CELL_SIZE) - across / 2,
        anchor.z.div_euclid(theme::CELL_SIZE) - across / 2,
    );
    let offset = IVec2::new(index as i32 % across, index as i32 / across);
    (home + offset) * theme::CELL_SIZE
}

pub fn cloud_height(anchor: IVec3) -> i32 {
    anchor.y + theme::RAIN_HEIGHT as i32
}

pub fn first_block_below(world: &VoxelWorld, x: i32, z: i32, cloud: i32) -> Option<IVec3> {
    (cloud - theme::RAIN_DEPTH..cloud)
        .rev()
        .map(|y| IVec3::new(x, y, z))
        .find(|&position| block_at(world, position) != BlockKind::Air)
}

fn stops_the_rain(world: &VoxelWorld, position: IVec3) -> bool {
    is_solid_at(world, position) || block_at(world, position) == BlockKind::Water
}

pub fn cell_floors(world: &VoxelWorld, anchor: IVec3) -> Vec<(f32, bool)> {
    let cloud = cloud_height(anchor);
    let bottom = cloud - theme::RAIN_DEPTH;
    (0..theme::CELLS_ACROSS * theme::CELLS_ACROSS)
        .map(|index| {
            let corner = cell_corner(anchor, index);
            let mut highest = bottom;
            let mut lowest = cloud;
            for x in corner.x..corner.x + theme::CELL_SIZE {
                for z in corner.y..corner.y + theme::CELL_SIZE {
                    let top = (bottom..cloud)
                        .rev()
                        .find(|&y| stops_the_rain(world, IVec3::new(x, y, z)))
                        .map_or(bottom, |y| y + 1);
                    highest = highest.max(top);
                    lowest = lowest.min(top);
                }
            }
            (highest as f32, highest - lowest <= theme::FLAT_STEP)
        })
        .collect()
}
