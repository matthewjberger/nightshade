use crate::components::{Avatar, Fit, Look};
use crate::data::CLIP_IDLE;
use crate::queries::models::pick_clips;
use crate::resources::{ModelKit, Models};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_gltf::Loading;
use nightshade_load::take;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab, spawn_prefab_with_skins};
use nightshade_scene::prelude::{ChildOf, LocalTransform};

pub fn collect(
    mut models: ResMut<Models>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let handles = models.handles.clone();
    let mut landed = Vec::new();
    for (index, handle) in handles.into_iter().enumerate() {
        let Some(handle) = handle else {
            continue;
        };
        let Some(outcome) = take(&mut *loading, handle) else {
            continue;
        };
        models.handles[index] = None;
        let Ok(mut import) = outcome else {
            models.failed[index] = true;
            continue;
        };
        let Some(prefab) = import.prefabs.first().cloned() else {
            models.failed[index] = true;
            continue;
        };
        let clips = pick_clips(&import.animations);
        let skins = import.skins.clone();
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        landed.push((
            index,
            ModelKit {
                prefab,
                clips,
                skins,
            },
        ));
    }
    for (index, kit) in landed {
        models.kits[index] = Some(kit);
    }
}

pub fn dress(
    models: Res<Models>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let wanted: Vec<(Entity, usize, f32)> = {
        storage
            .query_ref::<(&Look, &Fit, &Avatar)>()
            .iter()
            .filter(|(_, (look, _, avatar))| avatar.0.is_none() && models.kits[look.0].is_some())
            .map(|(entity, (look, fit, _))| (entity, look.0, fit.0))
            .collect()
    };
    if wanted.is_empty() {
        return;
    }
    let kits: Vec<Option<ModelKit>> = models.kits.clone();

    for (root, look, fit) in wanted {
        let Some(kit) = &kits[look] else {
            continue;
        };
        let animated = !kit.clips.is_empty();
        let child = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            if animated {
                spawn_prefab_with_skins(
                    &mut spawn,
                    &kit.prefab,
                    &kit.clips,
                    &kit.skins,
                    Vec3::zeros(),
                )
            } else {
                spawn_prefab(&mut spawn, &kit.prefab, Vec3::zeros())
            }
        };
        storage.set(child, ChildOf(root));
        if let Some(transform) = storage.get_mut::<LocalTransform>(child) {
            transform.translation = Vec3::zeros();
            transform.scale = Vec3::new(fit, fit, fit);
            transform.rotation = if animated {
                nalgebra_glm::quat_angle_axis(theme::MODEL_FACING, &Vec3::y())
            } else {
                nalgebra_glm::Quat::identity()
            };
        }
        if animated && let Some(player) = storage.get_mut::<AnimationPlayer>(child) {
            player.current_clip = Some(CLIP_IDLE);
            player.time = 0.0;
            player.speed = 1.0;
            player.looping = true;
            player.playing = true;
        }
        storage.set(root, Avatar(Some(child)));
    }
}
