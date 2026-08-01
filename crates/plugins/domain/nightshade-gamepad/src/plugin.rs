use crate::contract::Gamepad;
use crate::resources::GamepadBackend;
use crate::systems;
use nightshade::prelude::*;

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::input::Input => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(GamepadBackend::default());
        app.insert_resource(Gamepad::default());
        app.add_system(Stage::First, systems::input::gamepad_input_system);
        app.add_system(Stage::First, systems::input_mode::detect_input_mode_system);
    }
}
