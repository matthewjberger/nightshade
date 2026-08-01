use crate::contract::{OrreryCommand, OrreryCommands};
use crate::resources::{Mode, OrreryResources};

pub fn request(commands: &mut OrreryCommands, command: OrreryCommand) {
    commands.pending.push(command);
}

pub fn set_mode(resources: &mut OrreryResources, mode: Mode) {
    resources.flight.mode = mode;
}

pub fn record_signal(resources: &mut OrreryResources) {
    resources.flight.found = resources.flight.found.saturating_add(1);
}

pub fn set_signal_total(resources: &mut OrreryResources, total: u32) {
    resources.flight.total = total;
}
