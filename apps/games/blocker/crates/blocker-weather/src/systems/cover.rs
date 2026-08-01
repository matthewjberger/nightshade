use crate::data::settings::Settings;
use crate::queries::{cloud_height, first_block_below};
use crate::resources::{Overhead, Season, Settling, Sky};
use crate::theme;
use blocker_blocks::data::blocks::{BlockKind, Shape};
use blocker_blocks::queries::block_at;
use blocker_blocks::resources::VoxelWorld;
use nalgebra_glm::IVec3;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::between;
use nightshade_voxels::prelude::set_block;

pub fn settle_the_snow(
    time: Res<Time>,
    config: Res<Config<Settings>>,
    sky: Res<Sky>,
    season: Res<Season>,
    overhead: Res<Overhead>,
    mut world: ResMut<VoxelWorld>,
    mut random: ResMut<Random>,
    mut settling: ResMut<Settling>,
) {
    let allowed = config.toggle(Settings::SnowCover);
    let freezing = season.temperature <= theme::FREEZING;
    let covering = allowed && freezing && sky.snow > theme::COVER_SNOW;
    let melting = !allowed || !freezing;
    if !covering && !melting {
        settling.0 = 0.0;
        return;
    }
    settling.0 += time.delta_time.min(0.05) * config.number(Settings::SnowCoverRate);
    let over = overhead.0;
    let anchor = IVec3::new(
        over.x.floor() as i32,
        over.y.floor() as i32,
        over.z.floor() as i32,
    );
    let cloud = cloud_height(anchor);
    let reach = theme::COVER_REACH;
    while settling.0 >= 1.0 {
        settling.0 -= 1.0;
        let x = anchor.x + between(&mut random, -reach, reach).round() as i32;
        let z = anchor.z + between(&mut random, -reach, reach).round() as i32;
        let Some(top) = first_block_below(&world, x, z, cloud) else {
            continue;
        };
        let kind = block_at(&world, top);
        if melting {
            if kind == BlockKind::SnowLayer {
                set_block(&mut world, top, BlockKind::Air.index());
            }
            continue;
        }
        let bears_snow = kind.info().solid && kind.info().shape == Shape::Cube;
        if bears_snow {
            set_block(
                &mut world,
                top + IVec3::new(0, 1, 0),
                BlockKind::SnowLayer.index(),
            );
        }
    }
}
