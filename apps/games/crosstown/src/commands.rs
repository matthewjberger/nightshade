pub mod block;
pub mod fighters;
pub mod interface;
pub mod pickups;
pub mod textures;

use crate::contract::{CrosstownCommand, CrosstownCommands};

pub fn punch(commands: &mut CrosstownCommands) {
    commands.pending.push(CrosstownCommand::Punch);
}
