use crate::commands;
use crate::contract::{NightwireCommand, NightwireCommands, NightwireEvent};
use crate::resources::{NightwireResources, Screen};
use nightshade::prelude::*;
use nightshade_platform::prelude::Window;

pub fn apply(
    mut queue: ResMut<NightwireCommands>,
    mut resources: ResMut<NightwireResources>,
    mut window: ResMut<Window>,
    mut events: EventWriter<NightwireEvent>,
) {
    let pending = std::mem::take(&mut queue.pending);
    if pending.is_empty() {
        return;
    }

    for command in pending {
        let cards_before = resources.night.cards;
        let call_before = resources.night.call;
        let screen_before = resources.night.screen;

        match command {
            NightwireCommand::BeginShift => commands::begin_shift(&mut resources),
            NightwireCommand::Reply { index } => commands::answer_reply(&mut resources, index),
            NightwireCommand::RaiseCard { index } => commands::raise_card(&mut resources, index),
            NightwireCommand::Dispatch => commands::decide(&mut resources, true),
            NightwireCommand::Hold => commands::decide(&mut resources, false),
            NightwireCommand::Advance => commands::advance(&mut resources),
            NightwireCommand::ReturnToTitle => commands::return_to_title(&mut resources),
            NightwireCommand::LeaveStation => window.should_exit = true,
        }

        for (index, before) in cards_before.iter().enumerate() {
            if resources.night.cards[index] && !before {
                events.send(NightwireEvent::CardFound { index });
            }
        }
        if resources.night.call != call_before {
            events.send(NightwireEvent::CallClosed { index: call_before });
        }
        if resources.night.screen == Screen::Over && screen_before != Screen::Over {
            events.send(NightwireEvent::ShiftEnded);
        }
    }
}
