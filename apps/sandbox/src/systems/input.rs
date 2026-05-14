use crate::state::{Action, Sandbox, Screen};
use nightshade::prelude::*;

pub fn handle_global(sandbox: &mut Sandbox, world: &World) {
    let escape_pressed = world.resources.input.keyboard.just_pressed(KeyCode::Escape);
    let start_pressed = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::Start);

    match sandbox.screen {
        Screen::Title => {
            if escape_pressed {
                sandbox.pending_actions.push(Action::Quit);
            }
        }
        Screen::InGame => {
            if escape_pressed || start_pressed {
                sandbox.pending_actions.push(Action::PauseGame);
            }
            if world.resources.input.keyboard.just_pressed(KeyCode::KeyR) {
                sandbox.pending_actions.push(Action::ResetPlayer);
            }
        }
        Screen::Paused => {
            if escape_pressed || start_pressed {
                sandbox.pending_actions.push(Action::ResumeGame);
            }
        }
    }
}
