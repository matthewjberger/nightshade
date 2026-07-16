use crate::ecs::Screen;
use crate::state::Sandbox;
use crate::systems::shell;
use crate::systems::world::player;
use nightshade::prelude::*;

pub fn handle_global(sandbox: &mut Sandbox, input: Res<Input>, world: &mut World) {
    if shell::is_capturing_input(&sandbox.shell) {
        return;
    }
    let escape = input.keyboard.just_pressed(KeyCode::Escape);
    let start = world
        .res::<nightshade::plugins::gamepad::Gamepad>()
        .just_pressed_buttons
        .contains(&gilrs::Button::Start);
    let r = input.keyboard.just_pressed(KeyCode::KeyR);
    let sandbox_world = &mut sandbox.sandbox_world;

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
