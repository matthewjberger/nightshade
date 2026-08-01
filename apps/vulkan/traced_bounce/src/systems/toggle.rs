use crate::queries;
use crate::resources::Bounce;
use nightshade::prelude::*;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_platform::prelude::*;

pub fn take_the_ask(input: Res<Input>, mut bounce: ResMut<Bounce>) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        bounce.enabled = !bounce.enabled;
        bounce.asked = true;
    }
}

pub fn settle_the_grid(mut bounce: ResMut<Bounce>, mut settings: ResMut<RenderSettings>) {
    if !bounce.asked {
        return;
    }
    bounce.asked = false;
    settings.probe_volume = bounce.enabled.then(queries::probe_volume);
}
