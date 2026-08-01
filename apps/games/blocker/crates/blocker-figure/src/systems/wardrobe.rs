use crate::commands::attach_parts;
use crate::components::{Fit, Look};
use crate::data::characters::CHARACTERS;
use crate::resources::Wardrobe;
use crate::tags::Dressed;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade::prelude::*;
use nightshade_gltf::Loading;
use nightshade_load::{load_model_with_files, take};
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab};
use nightshade_render::commands::material_registry::material_registry_mutate_id;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::{ChildOf, LocalTransform};

pub fn demand_the_looks(mut wardrobe: ResMut<Wardrobe>, mut loading: ResMut<Loading>) {
    let in_flight = wardrobe
        .handles
        .iter()
        .filter(|handle| handle.is_some())
        .count();
    if in_flight >= theme::WARDROBE_IN_FLIGHT {
        return;
    }
    let mut budget = theme::WARDROBE_IN_FLIGHT - in_flight;
    for (index, character) in CHARACTERS.iter().enumerate() {
        if budget == 0 {
            break;
        }
        if wardrobe.requested[index] || wardrobe.kits[index].is_some() || wardrobe.failed[index] {
            continue;
        }
        let handle = load_model_with_files(
            &mut loading,
            character.model.to_vec(),
            std::iter::once((character.skin_path.to_string(), character.skin.to_vec())),
        );
        wardrobe.handles[index] = Some(handle);
        wardrobe.requested[index] = true;
        budget -= 1;
    }
}

pub fn collect_the_looks(
    mut wardrobe: ResMut<Wardrobe>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let handles = wardrobe.handles.clone();
    for (index, handle) in handles.into_iter().enumerate() {
        let Some(handle) = handle else {
            continue;
        };
        let Some(outcome) = take(&mut loading, handle) else {
            continue;
        };
        wardrobe.handles[index] = None;
        let Ok(mut import) = outcome else {
            wardrobe.failed[index] = true;
            continue;
        };
        let Some(prefab) = import.prefabs.first().cloned() else {
            wardrobe.failed[index] = true;
            continue;
        };
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        wardrobe.kits[index] = Some(prefab);
    }
}

pub fn dress_the_figures(
    wardrobe: Res<Wardrobe>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let mut bodies: Vec<(Entity, usize, f32)> = storage
        .query_ref::<(&Look, &Fit)>()
        .iter()
        .filter(|(entity, (look, _))| {
            wardrobe.kits.get(look.0).is_some_and(|kit| kit.is_some())
                && !storage.has_tag_type::<Dressed>(*entity)
        })
        .map(|(entity, (look, fit))| (entity, look.0, fit.0))
        .collect();
    if bodies.is_empty() {
        return;
    }
    bodies.truncate(theme::WARDROBE_BUDGET);
    for (root, index, fit) in bodies {
        storage.add_tag_type::<Dressed>(root);
        let Some(prefab) = wardrobe.kits[index].clone() else {
            continue;
        };
        let child = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            spawn_prefab(&mut spawn, &prefab, Vec3::zeros())
        };
        storage.set(child, ChildOf(root));
        if let Some(transform) = storage.get_mut::<LocalTransform>(child) {
            transform.translation = Vec3::new(0.0, theme::FIGURE_LIFT, 0.0);
            transform.scale = Vec3::new(fit, fit, fit);
        }
        let mut below = vec![child];
        below.extend(parts.hierarchy.descendants(child));
        attach_parts(storage, root, &below);
        light_the_figure(storage, parts.materials, &below);
    }
}

fn light_the_figure(storage: &Storage, materials: &mut MaterialRegistry, parts: &[Entity]) {
    let mut ids: Vec<_> = Vec::new();
    for entity in parts {
        if let Some(material) = storage.get::<MaterialRef>(*entity)
            && !ids.contains(&material.id)
        {
            ids.push(material.id);
        }
    }
    for id in ids {
        material_registry_mutate_id(materials, id, |material| material.unlit = false);
    }
}
