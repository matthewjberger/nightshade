use crate::ecs::{SandboxResources, Screen};
use crate::systems::shell::{self, SandboxShellContext};
use crate::systems::world::player;
use nightshade::prelude::*;
use nightshade::shell::ShellState;

pub fn handle_global(
    sandbox_world: &mut SandboxResources,
    input: Res<Input>,
    shell_state: Res<ShellState<SandboxShellContext>>,
    world: &mut World,
) {
    if shell::is_capturing_input(&shell_state) {
        return;
    }
    let escape = input.keyboard.just_pressed(KeyCode::Escape);
    let start = world
        .res::<nightshade::plugins::gamepad::Gamepad>()
        .just_pressed_buttons
        .contains(&gilrs::Button::Start);
    let r = input.keyboard.just_pressed(KeyCode::KeyR);

    match current_state::<Screen>(world) {
        Screen::Title => {
            if escape {
                world.res_mut::<Window>().should_exit = true;
            }
        }
        Screen::InGame => {
            if escape || start {
                next_state(world, Screen::Paused);
            } else if r {
                player::reset(sandbox_world, world);
            }
        }
        Screen::Paused => {
            if escape || start {
                next_state(world, Screen::InGame);
            }
        }
    }
}
