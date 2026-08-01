use crate::systems::shell::commands;
use crate::systems::shell::state::{SandboxShellContext, ShellAction};
use nightshade_shell::resources::state::ShellState;

pub fn new_shell() -> ShellState<SandboxShellContext> {
    let mut shell = ShellState::new(SandboxShellContext::default());
    shell.height = 420.0;
    commands::engine::register(&mut shell);
    commands::register(&mut shell);
    shell
}

pub fn drain_queue(shell: &mut ShellState<SandboxShellContext>) -> Vec<ShellAction> {
    std::mem::take(&mut shell.context.queue)
}
