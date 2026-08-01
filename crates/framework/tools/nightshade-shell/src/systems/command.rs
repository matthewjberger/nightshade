use crate::resources::state::ShellState;
use nightshade::prelude::*;

pub fn shell_command_system<C: Send + Sync + 'static>(
    mut shell: ResMut<ShellState<C>>,
    mut commands: Commands,
) {
    if !shell.pending_enter {
        return;
    }
    shell.pending_enter = false;
    shell.execute_command(&mut commands);
}
