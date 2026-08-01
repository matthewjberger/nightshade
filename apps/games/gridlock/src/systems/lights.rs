use crate::components::{Cell, Glow};
use crate::data::settings::Settings;
use crate::queries::grid::in_ring;
use crate::resources::Streaming;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_render::components::light::Light;

pub fn dim(
    config: Res<Config<Settings>>,
    mut streaming: ResMut<Streaming>,
    mut lamps: Query<(&mut Light, &Glow, &Cell)>,
) {
    let Some(center) = streaming.center else {
        return;
    };
    let cells = config.whole(Settings::LightCells) as i32;
    if streaming.lit == Some((center, cells)) {
        return;
    }
    streaming.lit = Some((center, cells));
    lamps.for_each_mut(|_, (light, glow, cell)| {
        let wanted = if in_ring(cell.0, center, cells) {
            glow.0
        } else {
            0.0
        };
        if light.intensity != wanted {
            light.intensity = wanted;
        }
    });
}
