use crate::contract::{AnimationPlayer, ClipSet};
use nightshade_ecs::{Entity, HierarchyIndex, Storage};
use nightshade_render::components::skin::Skin;
use nightshade_scene::contract::Name;

pub fn skeleton_joints(
    storage: &Storage,
    hierarchy: &HierarchyIndex,
    root: Entity,
) -> Vec<(String, Entity)> {
    let mut joints: Vec<(String, Entity)> = Vec::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Some(skin) = storage.get::<Skin>(entity) {
            for joint in &skin.joints {
                if let Some(name) = storage.get::<Name>(*joint)
                    && !joints.iter().any(|(held, _)| held == &name.0)
                {
                    joints.push((name.0.clone(), *joint));
                }
            }
        }
        stack.extend_from_slice(hierarchy.children(entity));
    }
    joints
}

pub fn skeleton_bones(storage: &Storage, hierarchy: &HierarchyIndex, root: Entity) -> Vec<String> {
    skeleton_joints(storage, hierarchy, root)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

pub fn clip_set_from_player(player: &AnimationPlayer, name: impl Into<String>) -> ClipSet {
    ClipSet {
        name: name.into(),
        clips: player.clips.clone(),
    }
}
