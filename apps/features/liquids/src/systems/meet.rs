use crate::data::Block;
use crate::resources::VoxelWorld;
use nightshade::prelude::ResMut;
use nightshade_voxels::prelude::set_block;
use nightshade_voxels::queries::block_at;

pub fn set_the_stone(mut world: ResMut<VoxelWorld>) {
    world.fallen.clear();
    if world.met.is_empty() {
        return;
    }
    let met = std::mem::take(&mut world.met);
    for (_, other) in met {
        if world.blocks.is_liquid(block_at(&world, other)) {
            set_block(&mut world, other, Block::Obsidian.index());
        }
    }
}
