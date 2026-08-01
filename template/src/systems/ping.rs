use crate::contract::{TemplateCommand, TemplateCommands, TemplateEvent};
use nightshade::prelude::*;

pub fn answer_pings(
    mut commands: ResMut<TemplateCommands>,
    mut events: EventWriter<TemplateEvent>,
) {
    for command in std::mem::take(&mut commands.pending) {
        match command {
            TemplateCommand::Ping { message } => {
                events.send(TemplateEvent::Pong { message });
            }
        }
    }
}
