use crate::commands::farming::{set_wet, water};
use crate::components::{PlotIndex, Progress};
use crate::data::CropKind;
use crate::queries::crops::stage_of;
use crate::queries::sheet::standing;
use crate::resources::{Diorama, Farmland, Sheet, Weather};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_sprite::prelude::*;

pub fn grow_the_crops(
    time: Res<Time>,
    sheet: Res<Sheet>,
    farmland: Res<Farmland>,
    weather: Res<Weather>,
    mut diorama: ResMut<Diorama>,
    crops: Query<(&CropKind, &mut Progress, &PlotIndex, &mut Sprite)>,
) {
    let hours = time.delta_time * theme::DAY_SPEED + diorama.skipped_hours;
    diorama.skipped_hours = 0.0;
    let day = diorama.day;
    let raining = weather.raining;
    crops.for_each(|_entity, (kind, progress, plot, sprite)| {
        let watered = raining
            || farmland
                .plots
                .get(plot.0)
                .is_some_and(|plot| plot.watered_day == Some(day));
        if !watered {
            return;
        }
        let before = stage_of(progress.0);
        progress.0 = (progress.0 + hours / kind.grow_hours()).min(1.0);
        let after = stage_of(progress.0);
        if before != after {
            let (region, height) = standing(&sheet, kind.stages()[after]);
            sprite.region = region;
            sprite.size.y = height;
        }
    });
}

pub fn tend_the_plots(
    diorama: Res<Diorama>,
    weather: Res<Weather>,
    mut farmland: ResMut<Farmland>,
    mut commands: Commands,
) {
    let day = diorama.day;
    if farmland.dried_day != day {
        farmland.dried_day = day;
        for plot in &farmland.plots {
            if plot.watered_day != Some(day) {
                set_wet(&mut commands, plot, false);
            }
        }
    }
    if !weather.raining {
        return;
    }
    for index in 0..farmland.plots.len() {
        water(&mut commands, &mut farmland, index, day);
    }
}
