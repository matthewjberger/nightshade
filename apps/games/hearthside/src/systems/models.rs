use crate::components::{Facing, Needs, Preview, Route, Using};
use crate::data;
use crate::queries;
use crate::resources::Models;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::ResMut;
use nightshade::prelude::Vec3;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_gltf::Loading;
use nightshade_load::take;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab, spawn_prefab_with_skins};
use nightshade_scene::prelude::LocalTransform;

pub fn collect(
    mut models: ResMut<Models>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    if models.ready {
        return;
    }

    let handles = models.handles.clone();
    if handles.is_empty() {
        return;
    }

    for (index, handle) in handles.into_iter().enumerate() {
        if models.prefabs[index].is_some() {
            continue;
        }
        let Some(Ok(mut import)) = take(&mut *loading, handle) else {
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

    if !models.prefabs.iter().all(Option::is_some) {
        return;
    }

    let prefabs = models.prefabs.clone();
    for (index, prefab) in prefabs.iter().enumerate() {
        let Some(prefab) = prefab else {
            continue;
        };
        let entity = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            spawn_prefab(&mut spawn, prefab, data::PARKED)
        };
        storage.set(entity, Preview(index));
    }

    models.ready = true;
}

pub fn raise_dweller(
    mut models: ResMut<Models>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    if models.dweller_spawned {
        return;
    }
    let Some(handle) = models.dweller_handle else {
        return;
    };
    let Some(outcome) = take(&mut *loading, handle) else {
        return;
    };
    let Ok(mut import) = outcome else {
        models.dweller_spawned = true;
        return;
    };

    let clip = |name: &str| {
        import
            .animations
            .iter()
            .position(|entry| entry.name.eq_ignore_ascii_case(name))
    };
    let idle = clip(data::CLIP_IDLE);
    let walk = clip(data::CLIP_WALK);
    let sit = clip(data::CLIP_SIT);

    let clips = import.animations.clone();
    let skins = import.skins.clone();
    let Some(prefab) = import.prefabs.first().cloned() else {
        models.dweller_spawned = true;
        return;
    };

    let start = queries::cell_index(data::DWELLER_START_CELL.0, data::DWELLER_START_CELL.1);
    let at = queries::cell_center(start);

    let dweller = {
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        spawn_prefab_with_skins(&mut spawn, &prefab, &clips, &skins, at)
    };

    if let Some(transform) = storage.get_mut::<LocalTransform>(dweller) {
        transform.scale = Vec3::new(
            data::DWELLER_SCALE,
            data::DWELLER_SCALE,
            data::DWELLER_SCALE,
        );
    }
    if let Some(index) = idle
        && let Some(player) = storage.get_mut::<AnimationPlayer>(dweller)
    {
        player.play(index);
        player.looping = true;
        player.speed = 1.0;
    }

    storage.set(dweller, Needs([72.0, 58.0, 64.0, 50.0]));
    storage.set(dweller, Route(Vec::new()));
    storage.set(dweller, Using(None));
    storage.set(dweller, Facing(0.0));

    models.clip_idle = idle;
    models.clip_walk = walk;
    models.clip_sit = sit;
    models.dweller_spawned = true;
}
