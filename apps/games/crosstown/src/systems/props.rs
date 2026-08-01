use crate::components::{Piece, Solid};
use crate::data;
use crate::queries::{far_end, look, prop_spots};
use crate::resources::{Models, Ring};
use nightshade::prelude::*;
use nightshade_gltf::Loading;
use nightshade_load::take;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab};
use nightshade_scene::prelude::LocalTransform;

pub fn place(
    mut models: ResMut<Models>,
    mut loading: ResMut<Loading>,
    mut ring: ResMut<Ring>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let handles = models.handles.clone();
    for (index, handle) in handles.into_iter().enumerate() {
        if models.prefabs[index].is_some() {
            continue;
        }
        let Some(Ok(mut import)) = take(&mut loading, handle) else {
            continue;
        };
        let prefab = import.prefabs.first().cloned();
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        models.prefabs[index] = prefab;
    }

    let prefabs = models.prefabs.clone();
    if prefabs.iter().any(Option::is_none) {
        return;
    }
    models.ready = true;

    if !ring.built || ring.dressed == ring.piece {
        return;
    }
    let shape = look(&ring);
    let end = far_end(&ring);
    let stage = ring.piece;
    ring.dressed = stage;

    for (model, at, yaw, radius) in prop_spots(shape, end) {
        let Some(prefab) = &prefabs[model] else {
            continue;
        };
        let entity = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            spawn_prefab(&mut spawn, prefab, at)
        };
        let scale = if radius > 0.0 {
            data::LANE_PROP_SCALE
        } else {
            data::PROP_SCALE
        };
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
            transform.scale = Vec3::new(scale, scale, scale);
        }
        if radius > 0.0 {
            storage.set(entity, Solid(radius));
        }
        storage.set(entity, Piece(stage));
    }
}
