use crate::resources::Jarvis;
use crate::systems;
use nightshade::prelude::*;
use nightshade_openxr::contract::{XrHand, XrUiAnchor, XrUiPanel};
use nightshade_platform::prelude::*;

pub struct JarvisPlugin;

impl Plugin for JarvisPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Jarvis".to_string();
        app.insert_resource(Jarvis::default());
        app.insert_resource(XrUiPanel {
            enabled: true,
            anchor: XrUiAnchor::Head,
            distance: crate::theme::PANEL_DISTANCE,
            width: crate::theme::PANEL_WIDTH,
            pointer_hand: XrHand::Right,
        });
        app.add_system(
            Stage::Startup,
            (systems::setup::spawn_hangar, systems::setup::build_visor),
        );
        app.add_system(
            Stage::Update,
            (
                systems::flight::fly_suit,
                systems::drones::fly_drones,
                systems::weapons::fire_weapons,
                systems::controls::drive_controls,
                systems::visor::draw_visor,
            ),
        );
    }
}
