use crate::systems::shell::state::SandboxShellContext;
use nightshade::prelude::*;
use nightshade_shell::data::Command;

pub const GAMEPAD: Command<SandboxShellContext> = Command {
    name: "gamepad",
    description: "Report the active pad, its stick and trigger values, and the buttons held",
    usage: "gamepad",
    execute: gamepad,
};

fn gamepad(_args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let state = &context.readout.gamepad;
    format!(
        "connected: {}
just pressed this frame: {}
held: {}
left stick: {:.3}, {:.3}
right stick: {:.3}, {:.3}
triggers: {:.3}, {:.3}
",
        state.connected,
        if state.just_pressed.is_empty() {
            "none".to_string()
        } else {
            state.just_pressed.join(", ")
        },
        if state.held.is_empty() {
            "none".to_string()
        } else {
            state.held.join(", ")
        },
        state.left_stick.0,
        state.left_stick.1,
        state.right_stick.0,
        state.right_stick.1,
        state.triggers.0,
        state.triggers.1,
    )
}
