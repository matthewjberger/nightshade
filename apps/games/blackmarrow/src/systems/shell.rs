use crate::data::settings::Settings;
use crate::resources::{BlackmarrowResources, Console};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::*;
use nightshade_shell::ShellState;
use nightshade_shell::data::Command;

const SHELL_HEIGHT: f32 = 300.0;

pub fn new_shell() -> ShellState<Console> {
    let mut shell = ShellState::new(Console::default());
    shell.height = SHELL_HEIGHT;
    shell.registry.register(Command {
        name: "noclip",
        description: "Let the diver pass through the station, with a speed if you give one",
        usage: "noclip [speed]",
        execute: pass_through_walls,
    });
    shell.registry.register(Command {
        name: "god",
        description: "Let the pack bite without taking health, and keep that in the settings",
        usage: "god",
        execute: shrug_off_the_teeth,
    });
    shell
}

pub fn read_the_keys(
    input: Res<Input>,
    mut shell: ResMut<ShellState<Console>>,
    reader: EventReader<AppEvent>,
) {
    for event in reader.iter() {
        if let AppEvent::Keyboard { key, state } = event {
            nightshade_shell::shell_key(&mut shell, &input, *key, *state);
        }
    }
}

pub fn route_the_asks(
    mut shell: ResMut<ShellState<Console>>,
    mut game: ResMut<BlackmarrowResources>,
    mut config: ResMut<Config<Settings>>,
) {
    let console = &mut shell.context;
    if console.noclip_asked {
        console.noclip_asked = false;
        game.player.noclip = console.noclip;
    }
    if console.invulnerable_asked {
        console.invulnerable_asked = false;
        config.set_toggle(Settings::Invulnerable, console.invulnerable);
    }
    console.noclip = game.player.noclip;
    console.invulnerable = config.toggle(Settings::Invulnerable);
}

fn shrug_off_the_teeth(
    _arguments: &[&str],
    _commands: &mut Commands,
    console: &mut Console,
) -> String {
    console.invulnerable = !console.invulnerable;
    console.invulnerable_asked = true;
    if console.invulnerable {
        "god mode on".to_string()
    } else {
        "god mode off".to_string()
    }
}

fn pass_through_walls(
    arguments: &[&str],
    _commands: &mut Commands,
    console: &mut Console,
) -> String {
    if let Some(speed) = arguments.first().and_then(|text| text.parse::<f32>().ok()) {
        console.fly_speed = speed.max(0.1);
    }
    console.noclip = !console.noclip;
    console.noclip_asked = true;
    if console.noclip {
        format!("noclip on, {:.1} a second", console.fly_speed)
    } else {
        "noclip off".to_string()
    }
}
