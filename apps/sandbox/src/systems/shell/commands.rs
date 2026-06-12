use crate::systems::shell::state::SandboxShellContext;
use nightshade::shell::{Command, ShellState};

pub mod level;
pub mod player;
pub mod time;

pub const ALL: &[Command<SandboxShellContext>] = &[
    player::PLAYER_POS,
    player::PLAYER_TP,
    player::RESET,
    player::FLY,
    player::THIRD_PERSON,
    player::STRIDE_WHEEL,
    player::SPHERES,
    player::SKELETON,
    player::POSE,
    player::RAGDOLL,
    time::TIME,
    time::TIME_PAUSE,
    time::TIME_SPEED,
    level::SAVE_LEVEL,
];

pub fn register(shell: &mut ShellState<SandboxShellContext>) {
    for command in ALL {
        shell.registry.register(*command);
    }
}
