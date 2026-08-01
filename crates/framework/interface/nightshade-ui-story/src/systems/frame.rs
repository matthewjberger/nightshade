use crate::resources::{StoryCard, StoryStyle};
use nightshade::prelude::*;
use nightshade_story::prelude::{Play, Story, links, obey, say, understand, wrap};
use nightshade_ui::commands::widgets::world_layout::{ui_events, ui_set_text, ui_set_visible};
use nightshade_ui::commands::widgets::world_state::{ui_button_set_text, ui_text_input_set_value};
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame, UiEvent};

pub fn take_the_line(
    frame: Res<RetainedUiFrame>,
    story: Res<Story>,
    mut play: ResMut<Play>,
    card: Res<StoryCard>,
    mut commands: Commands,
) {
    let mut asked: Vec<String> = Vec::new();
    let mut pressed: Vec<usize> = Vec::new();
    for event in ui_events(&frame) {
        match event {
            UiEvent::TextInputSubmitted { entity, text } if Some(*entity) == card.input => {
                asked.push(text.clone());
            }
            UiEvent::ButtonClicked(entity) => {
                if let Some(index) = card.links.iter().position(|link| link == entity) {
                    pressed.push(index);
                }
            }
            _ => {}
        }
    }

    for line in asked {
        if let Some(input) = card.input {
            ui_text_input_set_value(&mut commands, input, "");
        }
        if line.trim().is_empty() {
            continue;
        }
        say(&mut play, format!("> {}", line.trim()));
        match understand(&story, &play, &line) {
            Ok(deed) => obey(&mut play, &story, deed),
            Err(refusal) => say(&mut play, refusal),
        }
    }

    for index in pressed {
        let Some(deed) = card.offered.get(index).copied() else {
            continue;
        };
        if let Some(label) = card.labels.get(index) {
            let spoken = format!("> {label}");
            say(&mut play, spoken);
        }
        obey(&mut play, &story, deed);
    }
}

pub fn show_the_story(
    story: Res<Story>,
    play: Res<Play>,
    style: Res<StoryStyle>,
    mut card: ResMut<StoryCard>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let mut told: Vec<String> = Vec::new();
    for paragraph in &play.told {
        if paragraph.is_empty() {
            continue;
        }
        told.extend(wrap(paragraph, style.columns));
        told.push(String::new());
    }
    let start = told.len().saturating_sub(style.lines);
    let tail = &told[start..];

    for slot in 0..card.lines.len() {
        let wanted = tail.get(slot).cloned().unwrap_or_default();
        if card.shown.get(slot) == Some(&wanted) {
            continue;
        }
        ui_set_text(&mut commands, card.lines[slot], &wanted);
        if let Some(held) = card.shown.get_mut(slot) {
            *held = wanted;
        }
    }

    let offered = if play.ended {
        Vec::new()
    } else {
        links(&story, &play)
    };
    for slot in 0..card.links.len() {
        let wanted = offered
            .get(slot)
            .map(|link| link.words.clone())
            .unwrap_or_default();
        let standing = !wanted.is_empty();
        if card.lit.get(slot) != Some(&standing) {
            ui_set_visible(&mut dirty, &mut commands, card.links[slot], standing);
            if let Some(held) = card.lit.get_mut(slot) {
                *held = standing;
            }
        }
        if card.labels.get(slot) != Some(&wanted) {
            ui_button_set_text(&mut commands, card.links[slot], &wanted);
            if let Some(held) = card.labels.get_mut(slot) {
                *held = wanted;
            }
        }
    }
    card.offered = offered.into_iter().map(|link| link.deed).collect();

    let asking = !play.ended;
    if card.asking != asking {
        ui_set_visible(&mut dirty, &mut commands, card.input, asking);
        card.asking = asking;
    }
}
