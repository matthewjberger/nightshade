use crate::commands::models::request_model;
use crate::components::{Fit, Look, Rig};
use crate::data::DOG_MODEL;
use crate::queries::models::pick_clips;
use crate::resources::{ModelKit, Models};
use crate::tags::Dressed;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_gltf::Loading;
use nightshade_kit::prelude::MODELS;
use nightshade_load::take;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab, spawn_prefab_with_skins};
use nightshade_scene::prelude::{ChildOf, LocalTransform};

fn eye(active_camera: &ActiveCamera, storage: &Storage) -> Vec3 {
    active_camera
        .0
        .and_then(|camera| storage.get::<LocalTransform>(camera))
        .map(|transform| transform.translation)
        .unwrap_or_default()
}

pub fn demand(
    mut models: ResMut<Models>,
    mut loading: ResMut<Loading>,
    active_camera: Res<ActiveCamera>,
    storage: &mut Storage,
) {
    let in_flight = models
        .handles
        .iter()
        .filter(|handle| handle.is_some())
        .count();
    if in_flight >= theme::MODELS_IN_FLIGHT {
        return;
    }
    let eye = eye(&active_camera, storage);
    let mut nearest: Vec<Option<f32>> = vec![None; models.requested.len()];
    {
        let requested = &models.requested;
        for (entity, (look, transform)) in storage.query_ref::<(&Look, &LocalTransform)>().iter() {
            if requested[look.0] || storage.has_tag_type::<Dressed>(entity) {
                continue;
            }
            let distance = (transform.translation - eye).magnitude_squared();
            let slot = &mut nearest[look.0];
            if slot.is_none_or(|best| distance < best) {
                *slot = Some(distance);
            }
        }
    }
    let mut wanted: Vec<(f32, usize)> = nearest
        .iter()
        .enumerate()
        .filter_map(|(index, distance)| distance.map(|distance| (distance, index)))
        .collect();
    wanted.sort_by(|left, right| left.0.total_cmp(&right.0));
    for (_, index) in wanted.into_iter().take(theme::MODELS_IN_FLIGHT - in_flight) {
        let handle = request_model(&mut loading, index);
        models.handles[index] = Some(handle);
        models.requested[index] = true;
    }
}

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
        let clips = pick_clips(&import.animations, MODELS[index].name == DOG_MODEL);
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
        models.settle[index] = theme::SETTLE_FRAMES;
    }
}

pub fn dress(
    mut models: ResMut<Models>,
    active_camera: Res<ActiveCamera>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    for settle in models.settle.iter_mut() {
        *settle = settle.saturating_sub(1);
    }
    let eye = eye(&active_camera, storage);
    let mut roots: Vec<(f32, Entity, usize, f32)> = {
        if models.kits.is_empty() {
            return;
        }
        storage
            .query_ref::<(&Look, &Fit, &LocalTransform)>()
            .iter()
            .filter(|(entity, (look, _, _))| {
                ((models.kits[look.0].is_some() && models.settle[look.0] == 0)
                    || models.failed[look.0])
                    && !storage.has_tag_type::<Dressed>(*entity)
            })
            .map(|(entity, (look, fit, transform))| {
                (
                    (transform.translation - eye).magnitude_squared(),
                    entity,
                    look.0,
                    fit.0,
                )
            })
            .collect()
    };
    if roots.is_empty() {
        return;
    }
    roots.sort_by(|left, right| left.0.total_cmp(&right.0));
    roots.truncate(theme::DRESS_BUDGET);

    let kits: Vec<Option<ModelKit>> = models.kits.clone();

    for (_, root, look, fit) in roots {
        storage.add_tag_type::<Dressed>(root);
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
        if animated {
            if let Some(player) = storage.get_mut::<AnimationPlayer>(child) {
                player.current_clip = Some(crate::data::CLIP_IDLE);
                player.time = 0.0;
                player.speed = 1.0;
                player.looping = true;
                player.playing = true;
            }
            if storage.has::<Rig>(root) {
                storage.set(root, Rig(Some(child)));
            }
        }
    }
}
