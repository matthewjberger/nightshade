use crate::components::{CardSlot, LineSlot, ReplySlot, SummarySlot};
use crate::data::CALLS;
use crate::queries::{
    advance_text, caller_text, card_text, clock_text, notebook_note, reply_text, summary_rows,
    transcript_rows,
};
use crate::resources::{CallPhase, NightwireResources};
use nightshade::prelude::*;
use nightshade_ui::prelude::{ui_set_text, ui_set_visible};
use nightshade_ui::resources::RetainedUiDirty;

pub fn refresh(
    mut resources: ResMut<NightwireResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut lines: Query<(&LineSlot,)>,
    mut replies: Query<(&ReplySlot,)>,
    mut cards: Query<(&CardSlot,)>,
    mut summaries: Query<(&SummarySlot,)>,
) {
    if resources.rows_dirty {
        resources.rows_dirty = false;
        let targets = transcript_rows(&resources.night);
        let mut rows: Vec<(Entity, usize)> = Vec::new();
        lines.for_each_mut(|entity, (slot,)| rows.push((entity, slot.index)));
        for (entity, index) in rows {
            let Some(target) = targets.get(index) else {
                continue;
            };
            if resources.mirror.rows[index] != *target {
                resources.mirror.rows[index] = target.clone();
                ui_set_text(&mut commands, entity, target);
            }
        }
    }

    if !resources.controls_dirty {
        return;
    }
    resources.controls_dirty = false;

    let handles = resources.ui;
    let phase = resources.night.phase;
    let can_dispatch = CALLS[resources.night.call].can_dispatch;

    ui_set_text(&mut commands, handles.clock, &clock_text(&resources.night));
    ui_set_text(
        &mut commands,
        handles.caller,
        &caller_text(&resources.night),
    );
    ui_set_text(
        &mut commands,
        handles.notebook_note,
        &notebook_note(&resources.night),
    );

    let mut reply_nodes: Vec<(Entity, usize)> = Vec::new();
    replies.for_each_mut(|entity, (slot,)| reply_nodes.push((entity, slot.index)));
    for (entity, index) in reply_nodes {
        let text = reply_text(&resources.night, index);
        ui_set_visible(&mut dirty, &mut commands, entity, !text.is_empty());
        if !text.is_empty() {
            ui_set_text(&mut commands, entity, &text);
        }
    }

    let mut card_nodes: Vec<(Entity, usize)> = Vec::new();
    cards.for_each_mut(|entity, (slot,)| card_nodes.push((entity, slot.index)));
    for (entity, index) in card_nodes {
        let text = card_text(&resources.night, index);
        ui_set_visible(&mut dirty, &mut commands, entity, !text.is_empty());
        if !text.is_empty() {
            ui_set_text(&mut commands, entity, &text);
        }
    }

    let deciding = phase == CallPhase::Deciding && can_dispatch;
    ui_set_visible(&mut dirty, &mut commands, handles.send_button, deciding);
    ui_set_visible(&mut dirty, &mut commands, handles.hold_button, deciding);
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.advance_button,
        phase == CallPhase::Closed,
    );
    if phase == CallPhase::Closed {
        ui_set_text(
            &mut commands,
            handles.advance_button,
            &advance_text(&resources.night),
        );
    }

    let summary = summary_rows(&resources.night);
    let mut summary_nodes: Vec<(Entity, usize)> = Vec::new();
    summaries.for_each_mut(|entity, (slot,)| summary_nodes.push((entity, slot.index)));
    for (entity, index) in summary_nodes {
        if let Some(text) = summary.get(index) {
            ui_set_text(&mut commands, entity, text);
        }
    }
}
