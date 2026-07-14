use crate::ecs::{SandboxResources, Screen};
use crate::systems::world::player;
use nightshade::prelude::*;

pub fn handle_global(sandbox_world: &mut SandboxResources, world: &mut World) {
    let escape = world.resources.input.keyboard.just_pressed(KeyCode::Escape);
    let start = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::Start);
    let r = world.resources.input.keyboard.just_pressed(KeyCode::KeyR);

    match current_state::<Screen>(world) {
        Screen::Title => {
            if escape {
                world.resources.window.should_exit = true;
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
