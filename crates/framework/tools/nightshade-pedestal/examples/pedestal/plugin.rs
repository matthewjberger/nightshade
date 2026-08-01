use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_pedestal::{Pedestal, PedestalPlugin};
use nightshade_render::prelude::{Atmosphere, RenderSettings};

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Pedestal {
            compressed: false,
            model: theme::MODEL,
            sky: theme::SKY,
            back: theme::CAMERA_BACK,
            yaw: theme::CAMERA_YAW,
            pitch: theme::CAMERA_PITCH,
        });
        app.add_system(Stage::Startup, settle_the_look);
        app.add_plugin(PedestalPlugin::titled(theme::TITLE));
    }
}

fn settle_the_look(mut settings: nightshade::ecs::system_param::ResMut<RenderSettings>) {
    settings.atmosphere = Atmosphere::Hdr;
    settings.bloom_enabled = true;
}
