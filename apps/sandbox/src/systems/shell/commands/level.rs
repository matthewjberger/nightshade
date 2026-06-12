use crate::systems::shell::state::{SandboxShellContext, ShellAction};
use nightshade::prelude::*;
use nightshade::shell::Command;

pub const SAVE_LEVEL: Command<SandboxShellContext> = Command {
    name: "save_level",
    description: "Save the current level to disk",
    usage: "save_level",
    execute: save_level,
};

fn save_level(_args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    context.queue.push(ShellAction::SaveLevel);
    "saving".to_string()
}
