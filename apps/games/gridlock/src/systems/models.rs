use crate::commands::models::request_model;
use crate::components::{Fit, Look, Playing, Rig};
use crate::data::districts::CLIP_IDLE;
use crate::data::scale::{MODELS_IN_FLIGHT, SETTLE_FRAMES};
use crate::data::settings::Settings;
use crate::queries::models::pick_clips;
use crate::queries::view::{eye, forward, view_weight};
use crate::resources::{ModelKit, Models};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_config::contract::Config;
use nightshade_gltf::Loading;
use nightshade_load::take;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab, spawn_prefab_with_skins};
use nightshade_render::commands::mesh_cache::mesh_cache_protect;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_scene::prelude::{ChildOf, LocalTransform};

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
    if in_flight >= MODELS_IN_FLIGHT {
        return;
    }
    let eye = eye(&active_camera, storage);
    let facing = forward(&active_camera, storage);
    let mut nearest: Vec<Option<f32>> = vec![None; models.requested.len()];
    {
        let requested = &models.requested;
        for (_, (look, transform)) in storage.query_ref::<(&Look, &LocalTransform)>().iter() {
            if requested[look.0] {
                continue;
            }
            let offset = transform.translation - eye;
            let distance = view_weight(offset, facing) * offset.magnitude_squared();
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
    for (_, index) in wanted.into_iter().take(MODELS_IN_FLIGHT - in_flight) {
        let handle = request_model(&mut loading, index);
        models.handles[index] = Some(handle);
        models.requested[index] = true;
    }
}

pub fn collect(
    config: Res<Config<Settings>>,
    mut models: ResMut<Models>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let budget = config.whole(Settings::ModelsPerFrame).max(1) as usize;
    let handles = models.handles.clone();
    let mut landed = Vec::new();
    for (index, handle) in handles.into_iter().enumerate() {
        if landed.len() >= budget {
            break;
        }
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
        let mesh_names: Vec<String> = import.meshes.keys().cloned().collect();
        let texture_names: Vec<String> = import
            .materials
            .iter()
            .flat_map(|material| material.texture_names().map(str::to_string))
            .collect();
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        for name in mesh_names {
            mesh_cache_protect(spawn.parts.meshes, name);
        }
        for name in texture_names {
            texture_cache_protect(spawn.parts.textures, name);
        }
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
        models.settle[index] = SETTLE_FRAMES;
    }
}

pub fn dress(
    mut models: ResMut<Models>,
    active_camera: Res<ActiveCamera>,
    config: Res<Config<Settings>>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let budget = config.number(Settings::DressMillis) / 1000.0;
    let started = Instant::now();
    for settle in models.settle.iter_mut() {
        *settle = settle.saturating_sub(1);
    }
    if models.kits.is_empty() {
        return;
    }
    let eye = eye(&active_camera, storage);
    let facing = forward(&active_camera, storage);
    let mut roots: Vec<(f32, Entity, usize, f32)> = storage
        .query_ref::<(&Look, &Fit, &LocalTransform)>()
        .iter()
        .filter(|(_, (look, _, _))| {
            (models.kits[look.0].is_some() && models.settle[look.0] == 0) || models.failed[look.0]
        })
        .map(|(entity, (look, fit, transform))| {
            let offset = transform.translation - eye;
            (
                view_weight(offset, facing) * offset.magnitude_squared(),
                entity,
                look.0,
                fit.0,
            )
        })
        .collect();
    if roots.is_empty() {
        return;
    }
    roots.sort_by(|left, right| left.0.total_cmp(&right.0));

    let kits: &[Option<ModelKit>] = &models.kits;

    for (_, root, look, fit) in roots {
        storage.remove_components::<(Look, Fit)>(root);
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
            transform.rotation = nalgebra_glm::Quat::identity();
        }
        if animated {
            if let Some(player) = storage.get_mut::<AnimationPlayer>(child) {
                player.current_clip = Some(CLIP_IDLE);
                player.time = 0.0;
                player.speed = 1.0;
                player.looping = true;
                player.playing = true;
            }
            storage.set(child, Playing(CLIP_IDLE));
            if storage.has::<Rig>(root) {
                storage.set(root, Rig(Some(child)));
            }
        }
        if started.elapsed().as_secs_f32() >= budget {
            break;
        }
    }
}
