use crate::components::InPhase;
use crate::data::{FOX_OFFSETS, FOX_SCALE};
use crate::queries::strip_root_motion;
use crate::resources::Model;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_graphics::prelude::*;

pub fn stand_the_foxes(
    mut model: ResMut<Model>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = model.asked else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut loading, handle) else {
        return;
    };
    model.asked = None;
    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("the fox did not load: {error}");
            return;
        }
    };

    model.walk = import
        .animations
        .iter()
        .position(|clip| clip.name.to_lowercase().contains("walk"));
    model.run = import
        .animations
        .iter()
        .position(|clip| clip.name.to_lowercase().contains("run"));

    let whole = import.animations.clone();
    let filtered = strip_root_motion(&import.animations);
    let skins = import.skins.clone();
    let Some(prefab) = import.prefabs.first().cloned() else {
        tracing::error!("the fox holds no prefab");
        return;
    };

    let mut spawn = nightshade_prefab::PrefabSpawn { storage, parts };
    nightshade_prefab::queue_model_load(&mut spawn, &mut import);

    let walk = model.walk;
    for (offset_x, clips, in_phase) in [
        (FOX_OFFSETS[0], whole, true),
        (FOX_OFFSETS[1], filtered, false),
    ] {
        let fox = nightshade_prefab::spawn_prefab_with_skins(
            &mut spawn,
            &prefab,
            &clips,
            &skins,
            Vec3::zeros(),
        );
        if let Some(transform) = spawn.storage.get_mut::<LocalTransform>(fox) {
            transform.translation = nalgebra_glm::vec3(offset_x, 0.0, 0.0);
            transform.scale = Vec3::new(FOX_SCALE, FOX_SCALE, FOX_SCALE);
        }
        if let Some(index) = walk
            && let Some(player) = spawn.storage.get_mut::<AnimationPlayer>(fox)
        {
            player.play(index);
            player.looping = true;
            player.speed = 1.0;
        }
        spawn.storage.set(fox, InPhase(in_phase));
    }
}
