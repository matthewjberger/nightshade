use crate::components::{Bone, Fit, Gait, Head, Limb, Look, Owner, Pace, Plate, Tilt};
use crate::queries::{head_node, joint_of, plate_above};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, HierarchyIndex, Storage};
use nightshade::prelude::{Commands, Query};
use nightshade_scene::prelude::{Name, Visibility};
use nightshade_text::prelude::{
    TextProperties, TextState, spawn_3d_billboard_text_with_properties,
};

pub fn attach_parts(storage: &mut Storage, body: Entity, parts: &[Entity]) {
    let mut limbs = Vec::new();
    let mut heads = Vec::new();
    for entity in parts {
        let Some(name) = storage.get::<Name>(*entity) else {
            continue;
        };
        if let Some(joint) = joint_of(name.0.as_str()) {
            limbs.push((*entity, joint));
        } else if head_node(name.0.as_str()) {
            heads.push(*entity);
        }
    }
    for (entity, joint) in limbs {
        storage.set(entity, Limb(joint.offset()));
        storage.set(entity, Bone(joint));
        storage.set(entity, Owner(body));
    }
    for entity in heads {
        storage.set(entity, Head(0.0));
        storage.set(entity, Tilt(0.0));
        storage.set(entity, Owner(body));
    }
}

pub fn dress_the_body(commands: &mut Commands, body: Entity, look: usize, fit: f32) {
    commands.set(body, Look(look));
    commands.set(body, Fit(fit));
    commands.set(body, Gait(0.0));
    commands.set(body, Pace(0.0));
}

pub fn plate_the_body(
    text_state: &mut TextState,
    commands: &mut Commands,
    body: Entity,
    text: &str,
    place: Vec3,
    properties: TextProperties,
) {
    let plate = spawn_3d_billboard_text_with_properties(
        text_state,
        commands,
        text,
        plate_above(place),
        properties,
    );
    commands.set(body, Plate(plate));
}

pub fn show_the_tree(
    hierarchy: &HierarchyIndex,
    commands: &mut Commands,
    seen: &mut Query<'_, (&Visibility,)>,
    root: Entity,
    visible: bool,
) {
    let mut pending = vec![root];
    while let Some(entity) = pending.pop() {
        let mut current = None;
        seen.for_entity(entity, |(visibility,)| current = Some(visibility.visible));
        if current != Some(visible) {
            commands.set(entity, Visibility { visible });
        }
        pending.extend(hierarchy.children(entity).iter().copied());
    }
}

pub fn advance_the_stride(
    gait: &mut Gait,
    pace: &mut Pace,
    speed: f32,
    moving: bool,
    animation: f32,
    delta_time: f32,
) {
    let wanted = if moving { 1.0 } else { 0.0 };
    pace.0 += (wanted - pace.0) * (theme::PACE_EASE * delta_time).clamp(0.0, 1.0);
    gait.0 += speed * delta_time * theme::GAIT_RATE * pace.0 * animation;
    if gait.0 > std::f32::consts::TAU {
        gait.0 -= std::f32::consts::TAU;
    }
}
