use crate::resources::{BoomstickResources, MODEL_COUNT};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::ResMut;
use nightshade_animation::contract::AnimationClip;
use nightshade_gltf::Loading;
use nightshade_load::{LoadHandle, Source};
use nightshade_prefab::{Prefab, PrefabSpawn};
use std::collections::HashMap;

const MODELS: [(&[u8], &[u8]); MODEL_COUNT] = [
    (
        include_bytes!("../../assets/models/blocky/blocky-character-a.glb"),
        include_bytes!("../../assets/models/blocky/Textures/texture-a.png"),
    ),
    (
        include_bytes!("../../assets/models/blocky/blocky-character-c.glb"),
        include_bytes!("../../assets/models/blocky/Textures/texture-c.png"),
    ),
    (
        include_bytes!("../../assets/models/blocky/blocky-character-j.glb"),
        include_bytes!("../../assets/models/blocky/Textures/texture-j.png"),
    ),
    (
        include_bytes!("../../assets/models/blocky/blocky-character-l.glb"),
        include_bytes!("../../assets/models/blocky/Textures/texture-l.png"),
    ),
];

const WANTED_CLIPS: [&str; 4] = ["idle", "walk", "attack-melee-right", "die"];

const SKIN_NAMES: [&str; 4] = [
    "Textures/texture-a.png",
    "Textures/texture-c.png",
    "Textures/texture-j.png",
    "Textures/texture-l.png",
];

pub fn request(loading: &mut Loading, resources: &mut BoomstickResources) {
    for (slot, (model, skin)) in MODELS.iter().enumerate() {
        resources.assets.loads[slot] = Some(start(loading, model, skin));
    }
}

fn start(loading: &mut Loading, model: &[u8], skin: &[u8]) -> LoadHandle {
    let mut files = HashMap::new();
    for name in SKIN_NAMES {
        files.insert(name.to_string(), skin.to_vec());
    }
    nightshade_load::load(
        loading,
        Source::Parts {
            head: model.to_vec(),
            files,
        },
    )
}

pub fn collect(
    mut boomstick_resources: ResMut<BoomstickResources>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let mut ready: Vec<(usize, Prefab, Vec<AnimationClip>)> = Vec::new();
    {
        let handles = boomstick_resources.assets.loads;
        for (slot, handle) in handles.into_iter().enumerate() {
            let Some(handle) = handle else {
                continue;
            };
            let Some(outcome) = nightshade_load::take(&mut *loading, handle) else {
                continue;
            };
            boomstick_resources.assets.loads[slot] = None;
            let Ok(mut import) = outcome else {
                continue;
            };
            let Some(prefab) = import.prefabs.first().cloned() else {
                continue;
            };
            let clips = pick_clips(&import.animations);
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            nightshade_prefab::queue_model_load(&mut spawn, &mut import);
            ready.push((slot, prefab, clips));
        }
    }

    if ready.is_empty() {
        return;
    }
    let state = &mut *boomstick_resources;
    for (slot, prefab, clips) in ready {
        state.assets.prefabs[slot] = Some(prefab);
        state.assets.clips[slot] = clips;
    }
}

fn pick_clips(available: &[AnimationClip]) -> Vec<AnimationClip> {
    let fallback = available.first();
    WANTED_CLIPS
        .iter()
        .filter_map(|wanted| {
            available
                .iter()
                .find(|clip| clip.name.eq_ignore_ascii_case(wanted))
                .or(fallback)
                .cloned()
        })
        .collect()
}
