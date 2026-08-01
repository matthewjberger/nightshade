use crate::components::ItemKind;
use crate::data::rules::BATTERY_LOW;
use crate::data::theme;
use crate::queries::{carrying, condition};
use crate::resources::{AshfallResources, Weapon};
use crate::tags::Flashlight;
use nightshade::prelude::*;
use nightshade_flashlight::queries::shining;
use nightshade_flashlight::{Battery, Lit};
use nightshade_ui::prelude::*;

pub fn update(
    mut resources: ResMut<AshfallResources>,
    mut commands: Commands,
    mut torches: Query<(&Battery, &Lit), WithTag<Flashlight>>,
) {
    let resources = &mut *resources;
    let handles = resources.ui;

    let mut charge = 0.0;
    let mut alight = false;
    torches.for_each_mut(|_entity, (battery, lit)| {
        charge = battery.0;
        alight = shining(lit);
    });

    if !resources.screen.is_in_town() {
        return;
    }

    let (label, color) = condition(resources.player.health);
    if resources.hud.condition != label {
        resources.hud.condition = label.to_string();
        if let Some(node) = handles.condition {
            ui_set_text(&mut commands, node, label);
            ui_set_color::<UiBase>(&mut commands, node, color);
        }
    }

    let weapon = match resources.player.weapon {
        Weapon::Pipe => "STEEL PIPE".to_string(),
        Weapon::Pistol => format!("HANDGUN   {}", resources.player.ammo),
    };
    write_if_changed(
        &mut commands,
        handles.weapon,
        &mut resources.hud.weapon,
        weapon,
    );

    let mut keys = Vec::new();
    if carrying(&resources.satchel, ItemKind::BrassKey) {
        keys.push("Brass Key");
    }
    if carrying(&resources.satchel, ItemKind::WardKey) {
        keys.push("Ward Key");
    }
    if carrying(&resources.satchel, ItemKind::BoneKey) {
        keys.push("Bone Key");
    }
    if carrying(&resources.satchel, ItemKind::TownMap) {
        keys.push("Town Map");
    }
    write_if_changed(
        &mut commands,
        handles.keys,
        &mut resources.hud.keys,
        keys.join("   "),
    );

    let battery = {
        let shown = charge.round() as i32;
        if alight {
            format!("CELL   {shown}")
        } else {
            format!("cell   {shown}")
        }
    };
    if resources.hud.battery != battery {
        if let Some(node) = handles.battery {
            ui_set_text(&mut commands, node, &battery);
            let color = if charge <= BATTERY_LOW {
                theme::BATTERY_LOW_COLOR
            } else {
                theme::TEXT_DIM
            };
            ui_set_color::<UiBase>(&mut commands, node, color);
        }
        resources.hud.battery = battery;
    }

    let message = if resources.run.clock < resources.run.message_until {
        resources.run.message.clone()
    } else {
        String::new()
    };
    write_if_changed(
        &mut commands,
        handles.message,
        &mut resources.hud.message,
        message,
    );

    let prompt = resources.run.prompt.clone();
    write_if_changed(
        &mut commands,
        handles.prompt,
        &mut resources.hud.prompt,
        prompt,
    );

    let stance = {
        let posture = if resources.player.crouching {
            "CROUCHED"
        } else if resources.player.moving {
            "MOVING"
        } else {
            "STILL"
        };
        let light = if alight { "LIGHT ON" } else { "DARK" };
        let hands = if resources.player.holding.is_some() {
            "   HANDS FULL"
        } else {
            ""
        };
        format!("{posture}   {light}{hands}")
    };
    write_if_changed(
        &mut commands,
        handles.stance,
        &mut resources.hud.stance,
        stance,
    );

    let radio = if resources.run.radio > 0.05 {
        let count = (resources.run.radio * 12.0).ceil() as usize;
        let tick = (resources.run.clock * 19.0) as usize;
        let glyphs: String = (0..count)
            .map(|index| {
                if (index + tick).is_multiple_of(3) {
                    '/'
                } else {
                    '#'
                }
            })
            .collect();
        format!("{glyphs}  static")
    } else {
        String::new()
    };
    write_if_changed(
        &mut commands,
        handles.radio,
        &mut resources.hud.radio,
        radio,
    );
}

fn write_if_changed(
    commands: &mut Commands,
    node: Option<Entity>,
    cache: &mut String,
    text: String,
) {
    if *cache == text {
        return;
    }
    if let Some(node) = node {
        ui_set_text(commands, node, &text);
    }
    *cache = text;
}
