use std::collections::{HashMap, HashSet};

use nightshade::prelude::*;
use nightshade_graphics::resources::loading::TextureRecipe;
use nightshade_prefab::data::ModelImport;
use nightshade_render::components::morph::MorphWeights;
use nightshade_scene::contract::primitives::Name;

use crate::resources::{MorphInfo, ThumbnailImage};

const PREVIEW_SIZE: u32 = 96;

pub fn preview_sources(import: &ModelImport) -> Vec<(String, Vec<u8>)> {
    import
        .texture_plan
        .iter()
        .filter_map(|plan| {
            let image = match &plan.recipe {
                TextureRecipe::Direct { image } => Some(image.0),
                TextureRecipe::PackRgbA { rgb, .. } => rgb.as_ref().map(|image| image.0),
                TextureRecipe::PackRg { a, b } => a.as_ref().or(b.as_ref()).map(|image| image.0),
                _ => None,
            }?;
            let bytes = import.encoded_images.get(image as usize)?;
            Some((plan.name.clone(), bytes.clone()))
        })
        .collect()
}

pub fn decode_preview(name: String, bytes: &[u8]) -> Option<ThumbnailImage> {
    let image = image::load_from_memory(bytes).ok()?;
    let thumbnail = image.thumbnail(PREVIEW_SIZE, PREVIEW_SIZE).to_rgba8();
    let (width, height) = thumbnail.dimensions();
    Some(ThumbnailImage {
        key: name,
        pixels: thumbnail.into_raw(),
        width,
        height,
    })
}

pub fn morph_meshes(storage: &nightshade::ecs::Storage, entities: &[Entity]) -> Vec<MorphInfo> {
    entities
        .iter()
        .filter_map(|&entity| {
            let weights = storage.get::<MorphWeights>(entity)?;
            let name = storage
                .get::<Name>(entity)
                .map(|name| name.0.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| weights.mesh_name.clone());
            Some(MorphInfo {
                entity,
                name,
                weights: weights.weights.clone(),
            })
        })
        .collect()
}

pub fn ordered_entities(
    storage: &nightshade::ecs::Storage,
    roots: &[Entity],
    spawned: &HashSet<Entity>,
) -> Vec<Entity> {
    let mut children: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for &entity in spawned {
        if let Some(parent) = storage
            .get::<nightshade::ecs::ChildOf>(entity)
            .map(|link| link.0)
        {
            children.entry(parent).or_default().push(entity);
        }
    }
    for list in children.values_mut() {
        list.sort_by_key(|entity| entity.id);
    }

    let mut ordered = Vec::new();
    let mut seen: HashSet<Entity> = HashSet::new();
    let mut stack: Vec<Entity> = roots.iter().rev().copied().collect();
    while let Some(entity) = stack.pop() {
        if !spawned.contains(&entity) || !seen.insert(entity) {
            continue;
        }
        ordered.push(entity);
        if let Some(list) = children.get(&entity) {
            for child in list.iter().rev() {
                stack.push(*child);
            }
        }
    }
    let mut remaining: Vec<Entity> = spawned
        .iter()
        .copied()
        .filter(|entity| !seen.contains(entity))
        .collect();
    remaining.sort_by_key(|entity| entity.id);
    ordered.extend(remaining);
    ordered
}
