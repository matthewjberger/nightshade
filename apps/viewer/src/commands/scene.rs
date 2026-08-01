use std::collections::HashSet;

use nightshade::prelude::*;
use nightshade_graphics::prelude::Light;
use nightshade_scene::contract::primitives::{Name, Visibility};

use crate::commands::history::push;
use crate::commands::load::release_camera;
use crate::commands::selection::select_entity;
use crate::components::{Deleted, StowedLight};
use crate::queries::scene::{depth_of, is_visible, kind_of, model_cameras, parent_of};
use crate::queries::selection::group;
use crate::resources::{Edit, SceneRow, ViewerResources};

pub fn build_rows(viewer: &mut ViewerResources, storage: &nightshade::ecs::Storage) {
    let tracked: HashSet<Entity> = viewer
        .model
        .entities
        .iter()
        .copied()
        .filter(|entity| !storage.has::<Deleted>(*entity))
        .collect();
    viewer.model.cameras = model_cameras(storage, &viewer.model.entities);
    viewer.model.rows = viewer
        .model
        .entities
        .iter()
        .filter(|entity| tracked.contains(entity))
        .map(|&entity| {
            let parent = parent_of(storage, entity, &tracked);
            SceneRow {
                entity,
                parent,
                name: storage
                    .get::<Name>(entity)
                    .map(|name| name.0.clone())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| format!("Node {}", entity.id)),
                depth: depth_of(storage, entity, &tracked),
                kind: kind_of(storage, entity),
                visible: is_visible(storage, entity),
            }
        })
        .collect();
    viewer.view.tree_dirty = true;
}

fn delete_nodes(
    hierarchy: &nightshade::ecs::HierarchyIndex,
    storage: &mut nightshade::ecs::Storage,
    roots: &[Entity],
) -> Vec<Entity> {
    let mut targets: Vec<Entity> = Vec::new();
    for root in roots {
        for entity in std::iter::once(*root).chain(hierarchy.descendants(*root)) {
            if !targets.contains(&entity) && !storage.has::<Deleted>(entity) {
                targets.push(entity);
            }
        }
    }
    for entity in &targets {
        stow(storage, *entity);
    }
    targets
}

fn stow(storage: &mut nightshade::ecs::Storage, entity: Entity) {
    let visible = is_visible(storage, entity);
    if let Some(light) = storage.get::<Light>(entity).cloned() {
        storage.remove::<Light>(entity);
        storage.set(entity, StowedLight(light));
    }
    if let Some(visibility) = storage.get_mut::<Visibility>(entity) {
        visibility.visible = false;
    }
    storage.set(entity, Deleted(visible));
}

pub fn restore_deleted(storage: &mut nightshade::ecs::Storage, entities: &[Entity]) {
    for entity in entities {
        if !storage.is_alive(*entity) {
            continue;
        }
        let Some(Deleted(visible)) = storage.get::<Deleted>(*entity).copied() else {
            continue;
        };
        storage.remove::<Deleted>(*entity);
        if let Some(visibility) = storage.get_mut::<Visibility>(*entity) {
            visibility.visible = visible;
        }
        if let Some(StowedLight(light)) = storage.get::<StowedLight>(*entity).cloned() {
            storage.remove::<StowedLight>(*entity);
            storage.set(*entity, light);
        }
    }
}

pub fn delete_nodes_again(storage: &mut nightshade::ecs::Storage, entities: &[Entity]) {
    for entity in entities {
        if storage.is_alive(*entity) && !storage.has::<Deleted>(*entity) {
            stow(storage, *entity);
        }
    }
}

pub fn set_visible(
    viewer: &mut ViewerResources,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    visible: bool,
) {
    let mut targets = hierarchy.descendants(entity);
    targets.insert(0, entity);
    for target in targets {
        if storage.has::<Deleted>(target) {
            continue;
        }
        if let Some(visibility) = storage.get_mut::<Visibility>(target) {
            visibility.visible = visible;
        }
        if let Some(row) = viewer
            .model
            .rows
            .iter_mut()
            .find(|row| row.entity == target)
        {
            row.visible = visible;
        }
    }
    if viewer.selection.selected == Some(entity) {
        viewer.selection.visible = visible;
    }
    viewer.view.tree_visibility_dirty = true;
}

pub fn set_node_visible(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    visible: bool,
) {
    let hierarchy = std::mem::take(stage.hierarchy);
    set_visible(stage.viewer, &hierarchy, storage, entity, visible);
    *stage.hierarchy = hierarchy;
}

pub fn toggle_selected_visible(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let visible = stage.viewer.selection.visible;
    let members = group(stage.viewer, storage);
    for entity in members {
        set_node_visible(stage, storage, entity, !visible);
    }
}

pub fn toggle_isolate(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let isolated = stage.viewer.model.isolated;
    let roots = if isolated.is_some() {
        Vec::new()
    } else {
        group(stage.viewer, storage)
    };
    let hierarchy = std::mem::take(stage.hierarchy);
    isolate(stage.viewer, &hierarchy, storage, &roots);
    *stage.hierarchy = hierarchy;
}

pub fn rename(viewer: &mut ViewerResources, storage: &mut nightshade::ecs::Storage, name: String) {
    let Some(entity) = viewer.selection.selected else {
        return;
    };
    storage.set(entity, Name(name.clone()));
    viewer.selection.name = name.clone();
    if let Some(row) = viewer
        .model
        .rows
        .iter_mut()
        .find(|row| row.entity == entity)
    {
        row.name = name;
        viewer.view.tree_dirty = true;
    }
}

pub fn delete_selected(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let roots = group(stage.viewer, storage);
    if roots.is_empty() {
        return;
    }
    let hierarchy = std::mem::take(stage.hierarchy);
    let going = delete_nodes(&hierarchy, storage, &roots);
    *stage.hierarchy = hierarchy;
    release_camera(stage, &going);

    let viewer = &mut *stage.viewer;
    if viewer
        .model
        .isolated
        .is_some_and(|isolated| going.contains(&isolated))
    {
        viewer.model.isolated = None;
    }
    viewer.picking.hovered = None;
    push(&mut viewer.history, Edit::Delete(going));
    select_entity(stage, storage, None);
    let viewer = &mut *stage.viewer;
    build_rows(viewer, storage);
    stage.mesh_state.full_rebuild_needed = true;
}

pub fn isolate(
    viewer: &mut ViewerResources,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    storage: &mut nightshade::ecs::Storage,
    roots: &[Entity],
) {
    let keep: HashSet<Entity> = if roots.is_empty() {
        viewer.model.entities.iter().copied().collect()
    } else {
        let mut keep = HashSet::new();
        for root in roots {
            keep.extend(hierarchy.descendants(*root));
            keep.insert(*root);
            let mut current = *root;
            while let Some(parent) = storage
                .get::<nightshade::ecs::ChildOf>(current)
                .map(|link| link.0)
            {
                keep.insert(parent);
                current = parent;
            }
        }
        keep
    };
    for row in &mut viewer.model.rows {
        let visible = keep.contains(&row.entity);
        row.visible = visible;
        if let Some(visibility) = storage.get_mut::<Visibility>(row.entity) {
            visibility.visible = visible;
        }
    }
    if let Some(selected) = viewer.selection.selected {
        viewer.selection.visible = keep.contains(&selected);
    }
    viewer.model.isolated = roots.first().copied();
    viewer.view.tree_visibility_dirty = true;
}

pub fn register_spawned(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
) {
    {
        let viewer = &mut *stage.viewer;
        viewer.model.entities.push(entity);
        viewer.view.inspector_open = true;
        viewer.view.add_open = false;
    }
    select_entity(stage, storage, Some(entity));
    let viewer = &mut *stage.viewer;
    build_rows(viewer, storage);
}

pub fn apply_variant(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    name: Option<String>,
) {
    use nightshade_render::components::material::{MaterialRef, MaterialVariants};

    let registry = &mut *stage.materials;
    let updates: Vec<(Entity, nightshade_render::data::asset_id::MaterialId)> = storage
        .query_ref::<(&MaterialVariants, &MaterialRef)>()
        .iter()
        .filter_map(|(entity, (variants, material))| {
            let target = match name.as_deref() {
                None => variants.default_material_name.as_str(),
                Some(name) => variants.material_for_variant(name),
            };
            let target_id =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    registry, target,
                )?;
            (material.id != target_id).then_some((entity, target_id))
        })
        .collect();

    if updates.is_empty() {
        return;
    }
    for (entity, target) in updates {
        storage.set(entity, MaterialRef::new(target));
    }
    stage.mesh_state.full_rebuild_needed = true;
}
