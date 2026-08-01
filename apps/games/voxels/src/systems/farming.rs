use crate::components::{Block, Progress, Stage, Watered};
use crate::data::CropKind;
use crate::queries::{block_at, liquid_near, sheet_standing, stage_of};
use crate::resources::{Sheet, VoxelWorld, Weather};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_sprite::prelude::*;

pub fn grow(
    time: Res<Time>,
    sheet: Res<Sheet>,
    world: Res<VoxelWorld>,
    weather: Res<Weather>,
    mut commands: Commands,
    mut crops: Query<(
        &CropKind,
        &Block,
        &mut Progress,
        &mut Stage,
        &mut Watered,
        &mut Sprite,
    )>,
) {
    let delta_time = time.delta_time;
    let rained_on = weather.rain > 0.2;
    let mut uprooted = Vec::new();
    crops.for_each_mut(|entity, (kind, block, progress, stage, watered, sprite)| {
        if !block_at(&world, block.0).grows_crops() {
            uprooted.push(entity);
            return;
        }
        let info = kind.info();
        let pace = if watered.0 || rained_on {
            1.0
        } else {
            theme::DRY_GROW_MULTIPLIER
        };
        progress.0 = (progress.0 + delta_time * pace / info.grow_seconds).min(1.0);
        let reached = stage_of(progress.0);
        if reached != stage.0 {
            stage.0 = reached;
            watered.0 = rained_on || liquid_near(&world, block.0, theme::WATER_REACH, theme::WATER);
            let (region, height) = sheet_standing(&sheet, info.stages[reached]);
            sprite.region = region;
            sprite.size.y = height * 0.9;
        }
    });
    for entity in uprooted {
        commands.despawn(entity);
    }
}
