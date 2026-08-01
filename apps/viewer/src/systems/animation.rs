use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;

use crate::resources::{LayerView, ViewerResources};

pub fn mirror_animation(
    mut viewer: ResMut<ViewerResources>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let Some(&root) = viewer.model.roots.first() else {
        return;
    };
    let Some(player) = storage.get::<AnimationPlayer>(root) else {
        return;
    };
    viewer.animation.time = player.time;
    viewer.animation.playing = player.playing;
    viewer.animation.clip = player.current_clip;
    viewer.animation.speed = player.speed;
    viewer.animation.looping = player.looping;
    viewer.animation.duration = player
        .get_current_clip()
        .map(|clip| clip.duration)
        .unwrap_or(0.0);
    viewer.animation.blending = player.is_blending();

    let layers: Vec<LayerView> = player
        .layers
        .iter()
        .map(|layer| LayerView {
            clip: layer.clip_index,
            name: player
                .clips
                .get(layer.clip_index)
                .map(|clip| clip.name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| format!("Clip {}", layer.clip_index)),
            weight: layer.weight,
        })
        .collect();
    if layers != viewer.animation.layers {
        let rebuild = layers.len() != viewer.animation.layers.len()
            || layers
                .iter()
                .zip(&viewer.animation.layers)
                .any(|(next, held)| next.clip != held.clip || next.name != held.name);
        viewer.animation.layers = layers;
        viewer.view.layers_dirty |= rebuild;
    }
}
