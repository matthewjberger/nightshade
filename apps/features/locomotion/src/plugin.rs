use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers::{
    third_person_gamepad_input, third_person_mouse_input, third_person_update_transform,
};
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

use crate::resources::{LocomotionResources, Screen};
use crate::systems;

pub struct LocomotionPlugin;

impl Plugin for LocomotionPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Locomotion".to_string();
        app.insert_resource(LocomotionResources::default());
        app.add_plugin(nightshade_gltf::GltfPlugin);
        app.add_plugin(nightshade_animation::AnimationPlugin);
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(MenuPlugin::<Screen>::new(systems::screens::menu_style()));
        app.add_plugin(nightshade_gamepad::GamepadPlugin);
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Move>::new());
        app.add_system(Stage::Startup, systems::ground::lay_the_ground);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::arena::build);
        app.add_system(Stage::Update, systems::attach::attach_the_model);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::drive::drive),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::turn::turn_in_place),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::jump::jump),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::swap::swap_the_character),
        );
        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(Stage::Update, systems::screens::drive_the_menus);
        app.add_system(Stage::Update, systems::arena::spin);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::arena::collect),
        );
        app.add_system(Stage::Update, systems::readout::build);
        app.add_system(Stage::Update, systems::readout::refresh);
        app.add_system(Stage::Update, systems::readout::show_the_blend);
        app.add_system(Stage::Update, systems::readout::show_the_pad);
        app.add_system(
            Stage::Update,
            (
                third_person_mouse_input,
                third_person_gamepad_input,
                third_person_update_transform,
            ),
        );
    }
}
