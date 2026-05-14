use crate::ecs::{SandboxWorld, Screen};
use crate::systems::lifecycle;
use crate::systems::world::player;
use nightshade::prelude::*;

pub fn handle_global(sandbox_world: &mut SandboxWorld, world: &mut World) {
    let escape = world.resources.input.keyboard.just_pressed(KeyCode::Escape);
    let start = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::Start);
    let r = world.resources.input.keyboard.just_pressed(KeyCode::KeyR);
    let screen = sandbox_world.resources.screen.current;

    match screen {
        Screen::Title => {
            if escape {
                world.resources.window.should_exit = true;
            }
        }
        Screen::InGame => {
            if escape || start {
                lifecycle::enter(sandbox_world, world, Screen::Paused);
            } else if r {
                player::reset(sandbox_world, world);
            }
        }
        Screen::Paused => {
            if escape || start {
                lifecycle::enter(sandbox_world, world, Screen::InGame);
            }
        }
    }
}
