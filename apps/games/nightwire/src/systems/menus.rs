use crate::components::{CardSlot, ReplySlot};
use crate::contract::{NightwireCommand, NightwireCommands};
use crate::queries::card_is_open;
use crate::resources::{CallPhase, NightwireResources, Screen};
use nightshade::prelude::*;
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;

pub fn update(
    resources: Res<NightwireResources>,
    mut queue: ResMut<NightwireCommands>,
    frame: Res<RetainedUiFrame>,
    mut replies: Query<(&ReplySlot,)>,
    mut cards: Query<(&CardSlot,)>,
) {
    let clicks: Vec<Entity> = ui_button_clicks(&frame).collect();
    if clicks.is_empty() {
        return;
    }
    let handles = resources.ui;
    let night = &resources.night;

    for entity in clicks {
        let mut reply = None;
        replies.for_entity(entity, |(slot,)| reply = Some(slot.index));
        if let Some(index) = reply {
            if night.phase == CallPhase::Asking {
                queue.pending.push(NightwireCommand::Reply { index });
            }
            continue;
        }

        let mut card = None;
        cards.for_entity(entity, |(slot,)| card = Some(slot.index));
        if let Some(index) = card {
            if card_is_open(night, index) {
                queue.pending.push(NightwireCommand::RaiseCard { index });
            }
            continue;
        }

        let command = match night.screen {
            Screen::Title if entity == handles.begin_button => Some(NightwireCommand::BeginShift),
            Screen::Title if entity == handles.quit_button => Some(NightwireCommand::LeaveStation),
            Screen::Shift if entity == handles.send_button => Some(NightwireCommand::Dispatch),
            Screen::Shift if entity == handles.hold_button => Some(NightwireCommand::Hold),
            Screen::Shift if entity == handles.advance_button => Some(NightwireCommand::Advance),
            Screen::Over if entity == handles.again_button => Some(NightwireCommand::BeginShift),
            Screen::Over if entity == handles.over_title_button => {
                Some(NightwireCommand::ReturnToTitle)
            }
            _ => None,
        };
        if let Some(command) = command {
            queue.pending.push(command);
        }
    }
}
