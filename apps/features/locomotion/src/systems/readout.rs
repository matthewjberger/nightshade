use nightshade::prelude::*;
use nightshade_ui::prelude::*;

use crate::resources::LocomotionResources;

const PANEL_RECT: Rect = Rect {
    min: Vec2::new(16.0, 16.0),
    max: Vec2::new(340.0, 172.0),
};

pub fn build(mut locomotion: ResMut<LocomotionResources>, mut build: UiBuild) {
    if locomotion.readout.root.is_some() {
        return;
    }
    let mut tree = UiTreeBuilder::new(build.context());
    let panel = tree.add_floating_panel("locomotion_readout", "Locomotion", PANEL_RECT);
    let Some(content) = tree.panel_content(panel) else {
        return;
    };
    let mut speed = Entity::default();
    let mut blend = Entity::default();
    let mut clips = Entity::default();
    let mut pad = Entity::default();
    tree.in_parent(content, |tree| {
        speed = tree.add_label("");
        blend = tree.add_label("");
        clips = tree.add_label("");
        pad = tree.add_label("");
    });
    locomotion.readout.root = Some(panel);
    locomotion.readout.speed_text = Some(speed);
    locomotion.readout.blend_text = Some(blend);
    locomotion.readout.clip_text = Some(clips);
    locomotion.readout.pad_text = Some(pad);
}

pub fn refresh(mut locomotion: ResMut<LocomotionResources>, mut build: UiBuild) {
    let locomotion = &mut *locomotion;
    let Some(speed_text) = locomotion.readout.speed_text else {
        return;
    };
    let Some(blend_text) = locomotion.readout.blend_text else {
        return;
    };
    let character = &locomotion.character;
    let speed = character.velocity.norm();
    let shown = (
        (speed * 20.0).round() as i32,
        (character.forward_parameter * 20.0).round() as i32,
        (character.lateral_parameter * 20.0).round() as i32,
        i32::from(character.locked),
        locomotion.arena.collected as i32,
    );
    if locomotion.readout.shown == Some(shown) {
        return;
    }
    locomotion.readout.shown = Some(shown);
    let context = build.context();
    ui_set_text(
        context.commands,
        speed_text,
        &format!(
            "{speed:.2} m/s   rings {} of {}",
            locomotion.arena.collected, locomotion.arena.total
        ),
    );
    ui_set_text(
        context.commands,
        blend_text,
        &format!(
            "forward {:.2}   lateral {:.2}   {}",
            character.forward_parameter,
            character.lateral_parameter,
            if character.locked { "strafe" } else { "free" }
        ),
    );
}

pub fn show_the_blend(
    mut locomotion: ResMut<LocomotionResources>,
    mut build: UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    let locomotion = &mut *locomotion;
    let Some(clip_text) = locomotion.readout.clip_text else {
        return;
    };
    let Some(model) = locomotion.character.model else {
        return;
    };
    let Some(player) = storage.get::<nightshade_animation::contract::AnimationPlayer>(model) else {
        return;
    };
    let mut parts: Vec<String> = player
        .active_clips
        .iter()
        .filter(|active| active.weight > 0.02)
        .map(|active| {
            let name = player
                .clips
                .get(active.clip_index)
                .map(|clip| clip.name.as_str())
                .unwrap_or("?");
            format!("{name} {:.0}%", active.weight * 100.0)
        })
        .collect();
    if parts.is_empty() {
        parts.push("still".to_string());
    }
    let line = parts.join("  ");
    if locomotion.readout.clips_shown.as_deref() == Some(line.as_str()) {
        return;
    }
    locomotion.readout.clips_shown = Some(line.clone());
    let context = build.context();
    ui_set_text(context.commands, clip_text, &line);
}

pub fn show_the_pad(
    mut locomotion: ResMut<LocomotionResources>,
    pad: Res<nightshade_gamepad::contract::Gamepad>,
    mut build: UiBuild,
) {
    let locomotion = &mut *locomotion;
    let Some(pad_text) = locomotion.readout.pad_text else {
        return;
    };
    let line = if pad.connected {
        let mut held: Vec<String> = pad
            .held
            .iter()
            .map(|button| format!("{button:?}"))
            .collect();
        held.sort();
        let triggers = format!(
            "L{:.2} R{:.2}",
            pad.axis(nightshade_gamepad::GamepadAxis::LeftTrigger),
            pad.axis(nightshade_gamepad::GamepadAxis::RightTrigger)
        );
        if held.is_empty() {
            format!("pad {triggers}")
        } else {
            format!("pad {triggers}  {}", held.join(" "))
        }
    } else {
        "pad off".to_string()
    };
    if locomotion.readout.pad_shown.as_deref() == Some(line.as_str()) {
        return;
    }
    locomotion.readout.pad_shown = Some(line.clone());
    let context = build.context();
    ui_set_text(context.commands, pad_text, &line);
}
