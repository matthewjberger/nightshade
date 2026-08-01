use crate::data::{Block, building_block};
use crate::resources::{Digging, Plan, VoxelWorld};
use crate::theme;
use nalgebra_glm::IVec3;
use nightshade_voxels::prelude::{ColumnKey, Dig, write_block};

impl Dig for Digging {
    type Orders = Plan;
    type Order = ();
    type Tools = ();
    type Report = ();

    fn order(_orders: &mut Plan, _column: ColumnKey) {}

    fn dig(_order: (), _tools: &mut (), world: &mut VoxelWorld, column: ColumnKey) {
        for dx in 0..theme::CHUNK {
            for dz in 0..theme::CHUNK {
                let x = column[0] * theme::CHUNK + dx;
                let z = column[1] * theme::CHUNK + dz;
                for y in -theme::GROUND_DEPTH..=0 {
                    write_block(world, IVec3::new(x, y, z), Block::Ground.index());
                }
                for y in 1..=theme::HEIGHT {
                    if let Some(block) = building_block(x, y, z) {
                        write_block(world, IVec3::new(x, y, z), block.index());
                    }
                }
            }
        }
    }

    fn settle(
        _orders: &mut Plan,
        _world: &mut VoxelWorld,
        _column: ColumnKey,
        _report: (),
    ) -> Vec<ColumnKey> {
        Vec::new()
    }
}
