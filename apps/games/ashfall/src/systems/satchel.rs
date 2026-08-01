use crate::commands::{heal_player, post_message, stow, take_out};
use crate::components::ItemKind;
use crate::data::ask::Ask;
use crate::data::notes::NOTES;
use crate::data::rules::*;
use crate::data::theme::{TEXT_DIM, TEXT_PRIMARY, TEXT_RUST};
use crate::queries::{chosen_action, is_a_key, item_name, moved_across, moved_down, pairs_into};
use crate::resources::{AshfallResources, Screen, SlotAction, Weapon, WorldPhase};
use crate::systems::level::drop_pickup;
use crate::tags::Flashlight;
use nightshade::prelude::*;
use nightshade_flashlight::commands::recharge;
use nightshade_flashlight::{Battery, FlashlightSettings};
use nightshade_input::Actions;
use nightshade_ui::prelude::{UiBase, ui_set_color, ui_set_text};

pub fn update(
    actions: Res<Actions<Ask>>,
    settings: Res<FlashlightSettings>,
    mut resources: ResMut<AshfallResources>,
    mut commands: Commands,
    mut torches: Query<(&mut Battery,), WithTag<Flashlight>>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Inventory {
        return;
    }
    let step = moved_down(&actions);
    if step != 0 {
        let slots = resources.satchel.slots.len() as i32;
        let chosen = resources.satchel.chosen as i32 + step;
        resources.satchel.chosen = chosen.rem_euclid(slots) as usize;
    }
    let across = moved_across(&actions);
    if across != 0 {
        let count = SlotAction::ALL.len() as i32;
        let action = resources.satchel.action as i32 + across;
        resources.satchel.action = action.rem_euclid(count) as usize;
    }
    if actions.just_pressed(Ask::Confirm) {
        act(resources, &settings, &mut torches, &mut commands);
    }
    if (actions.just_pressed(Ask::Pause) || actions.just_pressed(Ask::ClosePockets))
        && resources.satchel.pairing.take().is_none()
    {
        resources.pending_screen = Some(Screen::Playing);
    }
    let mut charge = 0.0;
    torches.for_each_mut(|_entity, (battery,)| charge = battery.0);
    write(resources, charge, &mut commands);
}

fn act(
    resources: &mut AshfallResources,
    settings: &FlashlightSettings,
    torches: &mut Query<(&mut Battery,), WithTag<Flashlight>>,
    commands: &mut Commands,
) {
    let chosen = resources.satchel.chosen;
    if let Some(first) = resources.satchel.pairing.take() {
        join(resources, first, chosen);
        return;
    }
    let Some(kind) = resources.satchel.slots[chosen].kind else {
        return;
    };
    match chosen_action(resources.satchel.action) {
        SlotAction::Use => use_item(resources, settings, torches, kind, chosen),
        SlotAction::Examine => {
            resources.inspect.slot = chosen;
            resources.inspect.kind = Some(kind);
            resources.pending_screen = Some(Screen::Inspect);
        }
        SlotAction::Combine => {
            resources.satchel.pairing = Some(chosen);
            post_message(&mut resources.run, "With what?");
        }
        SlotAction::Drop => {
            if is_a_key(kind) || kind == ItemKind::TownMap {
                post_message(&mut resources.run, "You are not putting that down.");
                return;
            }
            take_out(&mut resources.satchel, chosen);
            drop_pickup(
                commands,
                resources.player.at,
                resources.player.upstairs,
                kind,
            );
            post_message(
                &mut resources.run,
                format!("You set the {} down at your feet.", item_name(kind)),
            );
        }
    }
}

fn join(resources: &mut AshfallResources, first: usize, second: usize) {
    if first == second {
        return;
    }
    let (Some(one), Some(two)) = (
        resources.satchel.slots[first].kind,
        resources.satchel.slots[second].kind,
    ) else {
        return;
    };
    let Some(made) = pairs_into(one, two) else {
        post_message(&mut resources.run, "Those two are nothing to each other.");
        return;
    };
    take_out(&mut resources.satchel, first);
    take_out(&mut resources.satchel, second);
    stow(&mut resources.satchel, made);
    resources.satchel.chosen = second;
    if made == ItemKind::BoneKey {
        resources.run.phase = WorldPhase::Otherworld;
    }
    post_message(
        &mut resources.run,
        format!("The two halves go together. A {}.", item_name(made)),
    );
}

fn use_item(
    resources: &mut AshfallResources,
    settings: &FlashlightSettings,
    torches: &mut Query<(&mut Battery,), WithTag<Flashlight>>,
    kind: ItemKind,
    slot: usize,
) {
    match kind {
        ItemKind::HealthDrink => {
            heal_player(&mut resources.player, HEALTH_DRINK);
            take_out(&mut resources.satchel, slot);
            post_message(&mut resources.run, "It tastes like pennies. You steady.");
        }
        ItemKind::Ammo => {
            resources.player.ammo += AMMO_BOX;
            take_out(&mut resources.satchel, slot);
            post_message(&mut resources.run, "Rounds into the handgun.");
        }
        ItemKind::Battery => {
            torches.for_each_mut(|_entity, (battery,)| {
                recharge(battery, settings, BATTERY_CHARGE);
            });
            take_out(&mut resources.satchel, slot);
            post_message(&mut resources.run, "A fresh cell. The beam hardens.");
        }
        ItemKind::TownMap => resources.pending_screen = Some(Screen::TownMap),
        ItemKind::Lockbox => {
            resources.inspect.slot = slot;
            resources.inspect.kind = Some(kind);
            resources.pending_screen = Some(Screen::Inspect);
        }
        ItemKind::Case => {
            resources.inspect.slot = slot;
            resources.inspect.kind = Some(kind);
            resources.pending_screen = Some(Screen::Inspect);
        }
        ItemKind::BoneHaft | ItemKind::BoneWard => post_message(
            &mut resources.run,
            "Half of a thing. It is looking for the rest of itself.",
        ),
        _ => post_message(
            &mut resources.run,
            "It only turns in the lock it was cut for.",
        ),
    }
}

pub fn write(resources: &AshfallResources, charge: f32, commands: &mut Commands) {
    let handles = resources.ui;
    for (index, node) in handles.inventory_slots.iter().enumerate() {
        let Some(node) = node else {
            continue;
        };
        let slot = resources.satchel.slots[index];
        let line = match slot.kind {
            Some(kind) if slot.count > 1 => format!("{}  x{}", item_name(kind), slot.count),
            Some(kind) => item_name(kind).to_string(),
            None => "empty".to_string(),
        };
        let here = index == resources.satchel.chosen;
        let text = if here {
            format!("> {line}")
        } else {
            format!("  {line}")
        };
        ui_set_text(commands, *node, &text);
        let color = match (here, slot.kind) {
            (true, _) => TEXT_RUST,
            (false, Some(_)) => TEXT_PRIMARY,
            (false, None) => TEXT_DIM,
        };
        ui_set_color::<UiBase>(commands, *node, color);
    }
    if let Some(node) = handles.inventory_actions
        && resources.satchel.pairing.is_some()
    {
        ui_set_text(commands, node, "Combine with which one?");
    } else if let Some(node) = handles.inventory_actions {
        let names: Vec<String> = SlotAction::ALL
            .iter()
            .enumerate()
            .map(|(index, action)| {
                if index == resources.satchel.action {
                    format!("[{}]", action.name())
                } else {
                    format!(" {} ", action.name())
                }
            })
            .collect();
        ui_set_text(commands, node, &names.join("   "));
    }
    if let Some(node) = handles.inventory_carried {
        let weapon = if resources.player.weapon == Weapon::Pistol {
            format!("Handgun, {} rounds", resources.player.ammo)
        } else {
            "Steel pipe".to_string()
        };
        let text = format!(
            "{weapon}   Cell at {} percent   {} of eight slots full",
            charge.round() as i32,
            resources
                .satchel
                .slots
                .iter()
                .filter(|slot| slot.kind.is_some())
                .count()
        );
        ui_set_text(commands, node, &text);
    }
    if let Some(node) = handles.inventory_notes {
        let titles: Vec<&str> = resources
            .run
            .found_notes
            .iter()
            .enumerate()
            .filter(|(_, got)| **got)
            .map(|(index, _)| NOTES[index].0)
            .collect();
        let text = if titles.is_empty() {
            "You have kept nothing to read.".to_string()
        } else {
            titles.join("\n")
        };
        ui_set_text(commands, node, &text);
    }
}
