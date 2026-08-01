use std::collections::HashSet;

use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_graphics::prelude::Light;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_scene::contract::primitives::{Name, Visibility};

use crate::components::Deleted;
use crate::data::RowKind;

pub fn model_cameras(
    storage: &nightshade::ecs::Storage,
    entities: &[Entity],
) -> Vec<(Entity, String)> {
    entities
        .iter()
        .filter(|entity| !storage.has::<Deleted>(**entity))
        .filter(|entity| storage.has::<Camera>(**entity))
        .enumerate()
        .map(|(index, entity)| {
            let name = storage
                .get::<Name>(*entity)
                .map(|name| name.0.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| format!("Camera {index}"));
            (*entity, name)
        })
        .collect()
}

pub fn is_visible(storage: &nightshade::ecs::Storage, entity: Entity) -> bool {
    storage
        .get::<Visibility>(entity)
        .is_none_or(|visibility| visibility.visible)
}

pub fn kind_of(storage: &nightshade::ecs::Storage, entity: Entity) -> RowKind {
    if storage.has::<RenderMesh>(entity) {
        RowKind::Mesh
    } else if storage.has::<Light>(entity) {
        RowKind::Light
    } else if storage.has::<Camera>(entity) {
        RowKind::Camera
    } else {
        RowKind::Node
    }
}

pub fn parent_of(
    storage: &nightshade::ecs::Storage,
    entity: Entity,
    tracked: &HashSet<Entity>,
) -> Option<Entity> {
    storage
        .get::<nightshade::ecs::ChildOf>(entity)
        .map(|link| link.0)
        .filter(|parent| tracked.contains(parent))
}

pub fn depth_of(
    storage: &nightshade::ecs::Storage,
    entity: Entity,
    tracked: &HashSet<Entity>,
) -> u32 {
    let mut depth = 0;
    let mut current = entity;
    while depth < 64 {
        let Some(parent) = parent_of(storage, current, tracked) else {
            break;
        };
        current = parent;
        depth += 1;
    }
    depth
}
