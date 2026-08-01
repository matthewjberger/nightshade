use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_pedestal::Pedestal;

pub struct ThinFilmPlugin;

impl Plugin for ThinFilmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Pedestal {
            compressed: false,
            model: theme::MODEL,
            sky: theme::SKY,
            back: theme::CAMERA_BACK,
            yaw: theme::CAMERA_YAW,
            pitch: theme::CAMERA_PITCH,
        });
        app.add_system(Stage::Startup, systems::setup::settle_the_look);
        app.add_plugin(nightshade_pedestal::PedestalPlugin::titled(theme::TITLE));
    }
}
