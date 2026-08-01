use crate::resources::Bag;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_inventory::commands::{add, select, select_next, swap, take};
use nightshade_inventory::queries::{held, in_hand};
use nightshade_inventory::{Selected, Slots, StackLimit};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    bag: Res<Bag>,
    limit: Res<StackLimit>,
    storage: &mut Storage,
) {
    let Some(holder) = bag.holder else {
        return;
    };
    let mut slots = storage.get::<Slots>(holder).cloned().unwrap_or_default();
    let mut selected = storage.get::<Selected>(holder).copied().unwrap_or_default();

    let offered = if input.keyboard.just_pressed(KeyCode::Digit1) {
        Some(theme::KINDS[0])
    } else if input.keyboard.just_pressed(KeyCode::Digit2) {
        Some(theme::KINDS[1])
    } else if input.keyboard.just_pressed(KeyCode::Digit3) {
        Some(theme::KINDS[2])
    } else {
        None
    };
    if let Some(kind) = offered {
        add(&mut slots, &limit, kind, theme::HANDFUL);
    }

    if input.keyboard.just_pressed(KeyCode::Space)
        && let Some(slot) = in_hand(&slots, &selected)
        && slot.count > 0
    {
        take(&mut slots, slot.kind, theme::HANDFUL);
    }

    if input.keyboard.just_pressed(KeyCode::ArrowRight) {
        select_next(&mut selected, &slots);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowLeft) && !slots.0.is_empty() {
        let back = (selected.0 + slots.0.len() - 1) % slots.0.len();
        select(&mut selected, &slots, back);
    }
    if input.keyboard.just_pressed(KeyCode::KeyX) && !slots.0.is_empty() {
        let next = (selected.0 + 1) % slots.0.len();
        swap(&mut slots, selected.0, next);
    }

    storage.set(holder, slots);
    storage.set(holder, selected);
}

pub fn show_the_bag(bag: Res<Bag>, mut readout: ResMut<Readout>, storage: &mut Storage) {
    let Some(holder) = bag.holder else {
        return;
    };
    let slots = storage.get::<Slots>(holder).cloned().unwrap_or_default();
    let selected = storage.get::<Selected>(holder).copied().unwrap_or_default();
    let posts = bag.posts.clone();
    for (index, post) in posts.iter().enumerate() {
        let count = slots.0.get(index).map(|slot| slot.count).unwrap_or(0);
        if let Some(transform) = storage.get_mut::<LocalTransform>(*post) {
            transform.scale.y = 0.4 + count as f32 * 0.35;
            transform.translation.y = transform.scale.y * 0.5;
            transform.scale.x = if index == selected.0 { 1.8 } else { 1.4 };
            transform.scale.z = transform.scale.x;
        }
    }

    let hand = in_hand(&slots, &selected)
        .map(|slot| {
            format!(
                "slot {} holds {} of kind {}",
                selected.0, slot.count, slot.kind
            )
        })
        .unwrap_or_else(|| "the hand is empty".to_string());
    readout.notes.clear();
    readout.notes.push(hand);
    for kind in theme::KINDS {
        let carried = held(&slots, kind);
        readout.notes.push(format!("kind {kind}: {carried}"));
    }
}
