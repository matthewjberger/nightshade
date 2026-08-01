use crate::data::Settings;
use crate::resources::SdfArguments;
use crate::theme;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::*;
use nightshade_sdf::contract::render_data::{SdfDebugView, SdfMarchScale};

pub fn cycle(input: Res<Input>, mut view: ResMut<SdfDebugView>) {
    if input.keyboard.just_pressed(KeyCode::Tab) {
        *view = view.next();
    }
}

pub fn start(
    arguments: Res<SdfArguments>,
    config: Res<Config<Settings>>,
    mut scale: ResMut<SdfMarchScale>,
) {
    scale.0 = arguments
        .scale
        .unwrap_or(config.number(Settings::MarchScale))
        .clamp(0.25, 1.0);
}

pub fn rescale(input: Res<Input>, mut scale: ResMut<SdfMarchScale>) {
    if !input.keyboard.just_pressed(KeyCode::KeyV) {
        return;
    }
    let current = theme::MARCH_SCALES
        .iter()
        .position(|candidate| (candidate - scale.0).abs() < 1.0e-3)
        .unwrap_or(0);
    scale.0 = theme::MARCH_SCALES[(current + 1) % theme::MARCH_SCALES.len()];
}
