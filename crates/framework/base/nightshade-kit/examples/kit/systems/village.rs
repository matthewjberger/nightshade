use crate::commands::{ask_for, lay_the_ground, stand};
use crate::data::Plan;
use crate::queries::{lay_out, named};
use crate::resources::{Village, Wanted};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_config::contract::Config;
use nightshade_load::take;
use nightshade_prefab::commands::{queue_model_load, spawn_prefab};
use nightshade_prefab::resources::PrefabSpawn;
use nightshade_readout::Readout;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn read_the_plan(config: Res<Config<Plan>>, mut village: ResMut<Village>) {
    let wanted = Wanted {
        houses: config.whole(Plan::Houses),
        seed: config.whole(Plan::Seed),
        street: config.whole(Plan::Street),
        square: config.whole(Plan::Square),
        spacing: config.whole(Plan::Spacing),
        trees: config.whole(Plan::Trees),
        rocks: config.whole(Plan::Rocks),
        reach: config.number(Plan::Reach),
        grass: config.color(Plan::Grass),
    };
    if village.shown == Some(wanted) {
        return;
    }
    village.shown = Some(wanted);
    village.spots = lay_out(&wanted);
    village.raised = 0;
    village.fresh = true;
}

pub fn lay_the_village(
    mut village: ResMut<Village>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let (fresh, wanted) = (village.fresh, village.shown.unwrap_or_default());
    if !fresh {
        return;
    }
    let ground = village.ground;
    let mut standing = std::mem::take(&mut village.standing);
    standing.push(ground);
    for entity in standing {
        if entity != nightshade::prelude::Entity::default() {
            storage.despawn_recursive(entity);
        }
    }
    let names = named(&village.spots);
    let mut waiting = Vec::new();
    for name in names {
        let handle = ask_for(&mut loading, name);
        waiting.push((handle, name));
    }
    let ground = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        lay_the_ground(commands, materials, wanted.reach + 4.0, wanted.grass)
    };
    village.waiting = waiting;
    village.ground = ground;
    village.fresh = false;
}

pub fn raise_what_landed(
    mut village: ResMut<Village>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let waiting = village.waiting.clone();
    let mut left = Vec::new();
    for (handle, name) in waiting {
        let Some(outcome) = take(&mut loading, handle) else {
            left.push((handle, name));
            continue;
        };
        let mut import = match outcome {
            Ok(import) => import,
            Err(error) => {
                tracing::error!("a piece of the kit did not load: {error}");
                continue;
            }
        };
        let places: Vec<(nalgebra_glm::Vec3, f32, f32)> = village
            .spots
            .iter()
            .filter(|spot| spot.model == name)
            .map(|spot| (spot.position, spot.yaw, spot.squeeze))
            .collect();
        let mut raised = Vec::new();
        {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            queue_model_load(&mut spawn, &mut import);
            for (position, yaw, squeeze) in &places {
                for prefab in &import.prefabs {
                    let entity = spawn_prefab(&mut spawn, prefab, *position);
                    raised.push((entity, *yaw, *squeeze));
                }
            }
        }
        for (entity, yaw, squeeze) in &raised {
            stand(storage, *entity, *yaw, *squeeze);
        }
        village.raised += raised.len();
        for (entity, _, _) in raised {
            village.standing.push(entity);
        }
    }
    village.waiting = left;
}

pub fn write_the_readout(village: Res<Village>, mut readout: ResMut<Readout>) {
    let wanted = village.shown.unwrap_or_default();
    readout.notes.clear();
    readout.notes.push(format!("seed {}", wanted.seed));
    readout.notes.push(format!("{} houses", wanted.houses));
    readout.notes.push(format!(
        "{} of {} pieces",
        village.raised,
        village.spots.len()
    ));
}
