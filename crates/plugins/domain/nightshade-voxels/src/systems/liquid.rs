use crate::commands::set_block;
use crate::contract::{AIR, FULL_LEVEL, Fill};
use crate::data;
use crate::queries::block_at;
use crate::resources::VoxelWorld;
use nalgebra_glm::IVec3;
use nightshade::prelude::ResMut;

pub fn spread_the_liquids(mut world: ResMut<VoxelWorld>) {
    world.ticks += 1;
    if world.wet.is_empty() {
        return;
    }
    let ticks = world.ticks;
    let mut pending = std::mem::take(&mut world.wet);
    if pending.len() > data::LIQUID_STEPS_PER_FRAME {
        world.wet = pending.split_off(data::LIQUID_STEPS_PER_FRAME);
    }
    let table = world.blocks.clone();
    let up = IVec3::new(0, 1, 0);
    for position in pending {
        let Some(fill) = table.fill(block_at(&world, position)) else {
            continue;
        };
        let Some(liquid) = table.liquids.get(fill.liquid as usize) else {
            continue;
        };
        if liquid.pace > 1 && !ticks.is_multiple_of(liquid.pace as u64) {
            world.wet.push(position);
            continue;
        }
        let under = block_at(&world, position - up);
        if table.replaceable(under) {
            match table.fill(under) {
                Some(other) if other.liquid != fill.liquid => {
                    world.met.push((position, position - up));
                }
                Some(other) if other.level >= FULL_LEVEL - 1 => {}
                _ => {
                    let poured = table.liquid_block(fill.liquid, FULL_LEVEL - 1);
                    set_block(&mut world, position - up, poured);
                }
            }
        } else if fill.level > 1 {
            let out = (fill.level - 1).min(liquid.reach);
            for offset in around() {
                let beside = block_at(&world, position + offset);
                if !table.replaceable(beside) {
                    continue;
                }
                match table.fill(beside) {
                    Some(other) if other.liquid != fill.liquid => {
                        world.met.push((position, position + offset));
                    }
                    Some(other) if other.level >= out => {}
                    _ => {
                        let poured = table.liquid_block(fill.liquid, out);
                        set_block(&mut world, position + offset, poured);
                    }
                }
            }
        }
        if fill.level != FULL_LEVEL && !fed(&world, position, fill) {
            set_block(&mut world, position, AIR);
        }
    }
}

fn fed(world: &VoxelWorld, position: IVec3, fill: Fill) -> bool {
    let same = |position: IVec3| {
        world
            .blocks
            .fill(block_at(world, position))
            .filter(|other| other.liquid == fill.liquid)
    };
    if same(position + IVec3::new(0, 1, 0)).is_some() {
        return true;
    }
    around()
        .into_iter()
        .any(|offset| same(position + offset).is_some_and(|other| other.level > fill.level))
}

fn around() -> [IVec3; 4] {
    [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ]
}
