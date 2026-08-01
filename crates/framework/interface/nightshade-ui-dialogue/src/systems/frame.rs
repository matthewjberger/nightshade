use crate::resources::DialogueCard;
use nightshade::prelude::*;
use nightshade_dialogue::Conversation;
use nightshade_dialogue::queries::{shown, talking};
use nightshade_ui::commands::widgets::world_layout::{ui_set_text, ui_set_visible};
use nightshade_ui::resources::RetainedUiDirty;

pub fn show_the_conversation(
    conversation: Res<Conversation>,
    mut card: ResMut<DialogueCard>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let speaking = talking(&conversation);
    ui_set_visible(&mut dirty, &mut commands, card.card, speaking);
    if !speaking {
        return;
    }
    if card.named != conversation.speaker {
        if let Some(name) = card.speaker {
            ui_set_text(&mut commands, name, &conversation.speaker);
        }
        card.named = conversation.speaker.clone();
    }
    let so_far = shown(&conversation);
    if card.shown == so_far {
        return;
    }
    if let Some(line) = card.line {
        ui_set_text(&mut commands, line, &so_far);
    }
    card.shown = so_far;
}
