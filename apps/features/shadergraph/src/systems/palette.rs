use crate::commands::{build_a_preset, clear_the_graph, place_a_kind};
use crate::components::{NodeKind, NodeSlot, NodeValue};
use crate::data::{Kind, PRESETS, note_of, takes_a_value, value_span_of};
use crate::resources::{Chosen, PaletteRows, Pictures, Tapping};
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_ui::prelude::{UiButtonData, UiDropdownData};
use nightshade_ui_nodegraph::prelude::{GraphGrip, Holding, NodeNote};

pub fn follow_the_presets(
    pictures: Res<Pictures>,
    mut rows: ResMut<PaletteRows>,
    mut chosen: ResMut<Chosen>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let Some(dropdown) = rows.presets else {
        return;
    };
    let Some(wanted) = storage
        .get::<UiDropdownData>(dropdown)
        .map(|data| data.selected_index)
    else {
        return;
    };
    if wanted == rows.shown {
        return;
    }
    let Some(preset) = PRESETS.get(wanted) else {
        return;
    };
    clear_the_graph(&mut commands, storage);
    build_a_preset(&mut commands, preset, &pictures);
    rows.shown = wanted;
    rows.placed = 0;
    chosen.0 = None;
}

pub fn follow_the_palette(
    mut rows: ResMut<PaletteRows>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let pressed: Vec<_> = rows
        .buttons
        .iter()
        .filter(|(button, _kind)| {
            storage
                .get::<UiButtonData>(*button)
                .is_some_and(|data| data.clicked)
        })
        .map(|(_button, kind)| *kind)
        .collect();

    for kind in pressed {
        let step = rows.placed % 6;
        let place = theme::FRESH_PLACE + theme::FRESH_STEP * step as f32;
        place_a_kind(&mut commands, kind, place);
        rows.placed += 1;
    }
}

pub fn follow_the_choice(grip: Res<GraphGrip>, mut chosen: ResMut<Chosen>, storage: &mut Storage) {
    let node = match grip.holding {
        Holding::Node(node) | Holding::Note(node) => node,
        _ => return,
    };
    if !storage.has::<NodeKind>(node) {
        return;
    }
    chosen.0 = Some(node);
}

pub fn drag_the_value(grip: Res<GraphGrip>, storage: &mut Storage) {
    let Holding::Note(node) = grip.holding else {
        return;
    };
    if grip.step.x == 0.0 {
        return;
    }
    let Some(kind) = storage.get::<NodeKind>(node).map(|kind| kind.0) else {
        return;
    };
    if !takes_a_value(kind) {
        return;
    }
    let Some(carried) = storage.get::<NodeValue>(node).map(|value| value.0) else {
        return;
    };
    let (low, high, _start) = value_span_of(kind);
    let rate = (high - low) * theme::DRAG_SHARE;
    let wanted = (carried + grip.step.x * rate).clamp(low, high);
    if let Some(value) = storage.get_mut::<NodeValue>(node) {
        value.0 = wanted;
    }
    if let Some(note) = storage.get_mut::<NodeNote>(node) {
        note.0 = note_of(kind, wanted);
    }
}

pub fn step_the_picture(
    grip: Res<GraphGrip>,
    mut pictures: ResMut<Pictures>,
    storage: &mut Storage,
) {
    let Holding::Note(node) = grip.holding else {
        pictures.nudge = 0.0;
        return;
    };
    let Some(kind) = storage.get::<NodeKind>(node).map(|kind| kind.0) else {
        return;
    };
    if kind != Kind::Picture {
        return;
    }
    pictures.nudge += grip.step.x;
    if pictures.nudge.abs() < theme::STEP_REACH {
        return;
    }
    let forward = pictures.nudge > 0.0;
    pictures.nudge = 0.0;

    let filled: Vec<u32> = pictures
        .held
        .iter()
        .enumerate()
        .filter(|(_slot, held)| held.is_some())
        .map(|(slot, _held)| slot as u32)
        .collect();
    if filled.is_empty() {
        return;
    }
    let held = storage
        .get::<NodeSlot>(node)
        .map(|slot| slot.0)
        .unwrap_or_default();
    let at = filled.iter().position(|slot| *slot == held).unwrap_or(0);
    let next = if forward {
        (at + 1) % filled.len()
    } else {
        (at + filled.len() - 1) % filled.len()
    };
    let wanted = filled[next];

    if let Some(slot) = storage.get_mut::<NodeSlot>(node) {
        slot.0 = wanted;
    }
    let named = pictures.held[wanted as usize]
        .as_ref()
        .map(|held| held.name.clone())
        .unwrap_or_else(|| "no picture".to_string());
    if let Some(note) = storage.get_mut::<NodeNote>(node) {
        note.0 = named;
    }
}

pub fn tap_the_field(grip: Res<GraphGrip>, mut tapping: ResMut<Tapping>) {
    tapping.tapped = None;
    let Holding::Note(node) = grip.holding else {
        tapping.node = None;
        tapping.travel = 0.0;
        return;
    };
    if tapping.node != Some(node) {
        tapping.node = Some(node);
        tapping.travel = 0.0;
    }
    tapping.travel += grip.step.x.abs() + grip.step.y.abs();
    if grip.letting_go && tapping.travel <= theme::TAP_REACH {
        tapping.tapped = Some(node);
    }
}
