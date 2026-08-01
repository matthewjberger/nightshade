use crate::contract::{TemplateCommand, TemplateCommands};

pub fn send_ping(commands: &mut TemplateCommands, message: impl Into<String>) {
    commands.pending.push(TemplateCommand::Ping {
        message: message.into(),
    });
}
