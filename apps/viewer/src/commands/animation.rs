use nightshade::prelude::Entity;
use nightshade_animation::contract::{AnimationLayer, AnimationPlayer};

pub fn blend_to_clip(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    index: usize,
) {
    let duration = stage.viewer.animation.blend_time;
    if duration > 0.0 {
        with_players(stage, storage, |player| {
            player.blend_to_in_phase(index, duration)
        });
    } else {
        with_players(stage, storage, |player| player.play(index));
    }
    let viewer = &mut *stage.viewer;
    viewer.animation.clip = Some(index);
    viewer.animation.playing = true;
}

pub fn add_animation_layer(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    clip: usize,
    weight: f32,
) {
    with_players(stage, storage, move |player| {
        if player.layers.iter().all(|layer| layer.clip_index != clip) {
            player
                .layers
                .push(AnimationLayer::new(clip).with_weight(weight));
        }
    });
    stage.viewer.view.layers_dirty = true;
}

pub fn toggle_playback(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let (playing, has_clip) = {
        let viewer = &mut *stage.viewer;
        (viewer.animation.playing, viewer.animation.clip.is_some())
    };
    if playing {
        with_players(stage, storage, |player| player.pause());
        stage.viewer.animation.playing = false;
    } else if has_clip {
        with_players(stage, storage, |player| player.resume());
        stage.viewer.animation.playing = true;
    } else {
        with_players(stage, storage, |player| player.play(0));
        let viewer = &mut *stage.viewer;
        viewer.animation.clip = Some(0);
        viewer.animation.playing = true;
        viewer.animation.duration = viewer
            .model
            .clips
            .first()
            .map(|clip| clip.duration)
            .unwrap_or(0.0);
    }
}

pub fn with_players(
    stage: &crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    action: impl Fn(&mut AnimationPlayer),
) {
    let roots = stage.viewer.model.roots.clone();
    for root in roots {
        if let Some(player) = storage.get_mut::<AnimationPlayer>(root) {
            action(player);
        }
    }
}

pub fn hold_clips(stage: &mut crate::resources::Stage<'_>, storage: &nightshade::ecs::Storage) {
    let roots = stage.viewer.model.roots.clone();
    let held = roots.iter().find_map(|root| {
        let player = storage.get::<AnimationPlayer>(*root)?;
        (!player.clips.is_empty())
            .then(|| nightshade_animation::clip_set_from_player(player, "held"))
    });
    if held.is_some() {
        stage.viewer.model.held_clips = held;
    }
}

pub fn apply_held_clips(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let notice = stage.viewer.incoming.notice.clone();
    let Some(set) = stage.viewer.model.held_clips.clone() else {
        crate::commands::notice::report(
            &notice,
            "No clips are held. Open a model that carries clips, then open the character to put them on.",
        );
        return;
    };
    let roots = stage.viewer.model.roots.clone();
    let Some(root) = roots.first().copied() else {
        crate::commands::notice::report(&notice, "There is no model open to put the clips on.");
        return;
    };

    let bones = {
        let hierarchy = &mut *stage.hierarchy;
        nightshade_animation::skeleton_bones(storage, hierarchy, root)
    };
    if bones.is_empty() {
        crate::commands::notice::report(&notice, "That model carries no skeleton.");
        return;
    }

    if storage.get::<AnimationPlayer>(root).is_none() {
        storage.set(root, AnimationPlayer::default());
        let mapping = {
            let hierarchy = &mut *stage.hierarchy;
            bone_entities(storage, hierarchy, root)
        };
        if let Some(player) = storage.get_mut::<AnimationPlayer>(root) {
            player.bone_name_to_entity = mapping;
        }
    }

    let Some(player) = storage.get_mut::<AnimationPlayer>(root) else {
        crate::commands::notice::report(&notice, "That model takes no clips.");
        return;
    };
    let report = nightshade_animation::clip_set_apply(player, &bones, &set);

    if report.is_complete() {
        crate::commands::notice::report(
            &notice,
            format!(
                "Put {} clips on this skeleton, {} channels.",
                report.clips, report.channels_resolved
            ),
        );
    } else {
        let missing = report
            .unresolved_bones
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        crate::commands::notice::report(
            &notice,
            format!(
                "Put {} clips on this skeleton, but {} channels found no joint. Missing: {}",
                report.clips, report.channels_dropped, missing
            ),
        );
    }

    refresh_clip_list(stage, storage, root);
}

fn bone_entities(
    storage: &nightshade::ecs::Storage,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    root: Entity,
) -> std::collections::HashMap<String, Entity> {
    let mut mapping = std::collections::HashMap::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Some(name) = storage.get::<nightshade_scene::contract::Name>(entity) {
            mapping.entry(name.0.clone()).or_insert(entity);
        }
        stack.extend_from_slice(hierarchy.children(entity));
    }
    mapping
}

fn refresh_clip_list(
    stage: &mut crate::resources::Stage<'_>,
    storage: &nightshade::ecs::Storage,
    root: Entity,
) {
    let clips: Vec<crate::resources::ClipInfo> = storage
        .get::<AnimationPlayer>(root)
        .map(|player| {
            player
                .clips
                .iter()
                .map(|clip| crate::resources::ClipInfo {
                    name: clip.name.clone(),
                    duration: clip.duration,
                })
                .collect()
        })
        .unwrap_or_default();
    let viewer = &mut *stage.viewer;
    viewer.model.clips = clips;
    viewer.view.tree_dirty = true;
}
