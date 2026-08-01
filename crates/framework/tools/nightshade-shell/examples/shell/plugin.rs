use crate::resources::Console;
use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_shell::data::Command;
use nightshade_shell::{ShellOverlayPlugin, ShellState};

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(new_shell());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(ShellOverlayPlugin::<Console>::default());
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Update, systems::play::read_the_keys);
        app.add_system(Stage::Update, systems::play::follow_the_console);
    }
}

fn new_shell() -> ShellState<Console> {
    let mut shell = ShellState::new(Console::default());
    shell.height = theme::SHELL_HEIGHT;
    shell.registry.register(Command {
        name: "spin",
        description: "Set how fast the body turns",
        usage: "spin <rate>",
        execute: set_the_spin,
    });
    shell.registry.register(Command {
        name: "stop",
        description: "Hold the body still",
        usage: "stop",
        execute: hold_the_body,
    });
    shell
}

fn set_the_spin(
    arguments: &[&str],
    _commands: &mut nightshade::prelude::Commands,
    console: &mut Console,
) -> String {
    let Some(rate) = arguments.first().and_then(|text| text.parse::<f32>().ok()) else {
        return "spin takes one number".to_string();
    };
    console.spin = rate.clamp(theme::SPIN_LOW, theme::SPIN_HIGH);
    format!("spin {:.2}", console.spin)
}

fn hold_the_body(
    _arguments: &[&str],
    _commands: &mut nightshade::prelude::Commands,
    console: &mut Console,
) -> String {
    console.spin = 0.0;
    "the body is still".to_string()
}
