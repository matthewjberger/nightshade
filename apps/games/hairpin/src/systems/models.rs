use crate::components::{Steer, Velocity, Yaw};
use crate::data::{CAR_START, CAR_YAW, MODEL_COUNT, TRUCK};
use crate::queries::{track_limits, track_tiles};
use crate::resources::HairpinResources;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_gltf::Loading;
use nightshade_load::take;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab};
use nightshade_scene::prelude::LocalTransform;

pub fn build(
    mut hairpin_resources: ResMut<HairpinResources>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    if hairpin_resources.track.built {
        return;
    }

    let handles = hairpin_resources.models.handles.clone();
    for (index, handle) in handles.into_iter().enumerate() {
        let Some(Ok(mut import)) = take(&mut *loading, handle) else {
            continue;
        };
        let prefab = import.prefabs.first().cloned();
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        hairpin_resources.models.prefabs[index] = prefab;
    }

    let prefabs = hairpin_resources.models.prefabs.clone();
    if prefabs.len() < MODEL_COUNT || prefabs.iter().any(Option::is_none) {
        return;
    }

    let tiles = track_tiles();
    for &(kind, at, yaw) in &tiles {
        let Some(tile) = &prefabs[kind] else { continue };
        let entity = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            spawn_prefab(&mut spawn, tile, at)
        };
        turn(storage, entity, yaw);
    }

    let Some(truck) = &prefabs[TRUCK] else { return };
    let car = {
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        spawn_prefab(&mut spawn, truck, CAR_START)
    };
    storage.set(car, Velocity::default());
    storage.set(car, Yaw(CAR_YAW));
    storage.set(car, Steer::default());
    turn(storage, car, CAR_YAW);

    let state = &mut *hairpin_resources;
    state.track.limits = track_limits(&tiles);
    state.track.car = Some(car);
    state.track.built = true;
}

fn turn(storage: &mut Storage, entity: Entity, yaw: f32) {
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    }
}
