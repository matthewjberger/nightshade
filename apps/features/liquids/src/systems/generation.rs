use crate::data::Block;
use crate::resources::{Digging, Noises, Seed, VoxelWorld};
use crate::theme;
use nalgebra_glm::IVec3;
use nightshade_voxels::prelude::{ColumnKey, Dig, write_block};
use noise::NoiseFn;

impl Dig for Digging {
    type Orders = Seed;
    type Order = u32;
    type Tools = Noises;
    type Report = Vec<IVec3>;

    fn order(orders: &mut Seed, _column: ColumnKey) -> u32 {
        orders.0
    }

    fn dig(
        seed: u32,
        noises: &mut Noises,
        world: &mut VoxelWorld,
        column: ColumnKey,
    ) -> Vec<IVec3> {
        if noises.seed != seed {
            *noises = Noises::new(seed);
        }
        let mut sources = Vec::new();
        for dx in 0..theme::CHUNK {
            for dz in 0..theme::CHUNK {
                let x = column[0] * theme::CHUNK + dx;
                let z = column[1] * theme::CHUNK + dz;
                let land = noises
                    .land
                    .get([x as f64 * theme::LAND_SCALE, z as f64 * theme::LAND_SCALE]);
                let heat = noises
                    .heat
                    .get([x as f64 * theme::HEAT_SCALE, z as f64 * theme::HEAT_SCALE]);
                let hot = heat > theme::HEAT_EDGE;
                let mut height = (theme::GROUND + land * theme::RELIEF).round() as i32;
                if hot {
                    height = height.max(theme::SEA_LEVEL + 1);
                }
                let top = if hot {
                    Block::Stone
                } else if height <= theme::SEA_LEVEL + 1 {
                    Block::Sand
                } else {
                    Block::Grass
                };
                for y in theme::FLOOR_Y..=height {
                    let block = if y < height - theme::SOIL_DEPTH {
                        Block::Stone
                    } else if y < height {
                        if hot { Block::Stone } else { Block::Dirt }
                    } else {
                        top
                    };
                    write_block(world, IVec3::new(x, y, z), block.index());
                }
                if !hot {
                    for y in height + 1..=theme::SEA_LEVEL {
                        write_block(world, IVec3::new(x, y, z), Block::WaterFive.index());
                    }
                }
                if heat > theme::HEAT_POOL {
                    let pool = IVec3::new(x, height + 1, z);
                    write_block(world, pool, Block::LavaFive.index());
                    sources.push(pool);
                }
            }
        }
        sources
    }

    fn settle(
        _orders: &mut Seed,
        world: &mut VoxelWorld,
        _column: ColumnKey,
        sources: Vec<IVec3>,
    ) -> Vec<ColumnKey> {
        world.wet.extend(sources);
        Vec::new()
    }
}
