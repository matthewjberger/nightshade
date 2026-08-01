use nightshade_camera::components::Camera;
use nightshade_graphics::prelude::Light;

use crate::commands::material::reset_to;
use crate::commands::scene::{build_rows, delete_nodes_again, restore_deleted};
use crate::commands::selection::{restore_transform, select_entity, write_camera, write_light};
use crate::data::MERGE_FRAMES;
use crate::resources::{Edit, History, TransformEdit};

pub fn push(history: &mut History, edit: Edit) {
    history.undone.clear();
    let frame = history.frame;
    if let Some((last, at)) = history.done.last_mut()
        && frame.saturating_sub(*at) <= MERGE_FRAMES
        && merge(last, &edit)
    {
        *at = frame;
        return;
    }
    history.done.push((edit, frame));
}

pub fn undo(stage: &mut crate::resources::Stage<'_>, storage: &mut nightshade::ecs::Storage) {
    let edit = stage.viewer.history.done.pop();
    if let Some((edit, _)) = edit {
        restore_edit(stage, storage, &edit, false);
        stage.viewer.history.undone.push(edit);
    }
}

pub fn redo(stage: &mut crate::resources::Stage<'_>, storage: &mut nightshade::ecs::Storage) {
    let edit = stage.viewer.history.undone.pop();
    if let Some(edit) = edit {
        restore_edit(stage, storage, &edit, true);
        let viewer = &mut *stage.viewer;
        let frame = viewer.history.frame;
        viewer.history.done.push((edit, frame));
    }
}

fn merge(last: &mut Edit, next: &Edit) -> bool {
    match (last, next) {
        (Edit::Transforms(held), Edit::Transforms(value))
            if held.len() == value.len()
                && held
                    .iter()
                    .zip(value)
                    .all(|(mine, theirs)| mine.entity == theirs.entity) =>
        {
            for (mine, theirs) in held.iter_mut().zip(value) {
                mine.after = theirs.after;
            }
            true
        }
        (
            Edit::Material { name, after, .. },
            Edit::Material {
                name: other,
                after: value,
                ..
            },
        ) if name == other => {
            *after = value.clone();
            true
        }
        (
            Edit::Light { entity, after, .. },
            Edit::Light {
                entity: other,
                after: value,
                ..
            },
        ) if entity == other => {
            *after = value.clone();
            true
        }
        (
            Edit::Camera { entity, after, .. },
            Edit::Camera {
                entity: other,
                after: value,
                ..
            },
        ) if entity == other => {
            *after = value.clone();
            true
        }
        _ => false,
    }
}

fn restore_edit(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    edit: &Edit,
    forward: bool,
) {
    match edit {
        Edit::Transforms(edits) => {
            for TransformEdit {
                entity,
                before,
                after,
            } in edits
            {
                let value = if forward { after } else { before };
                restore_transform(storage, *entity, value);
            }
        }
        Edit::Delete(entities) => {
            if forward {
                delete_nodes_again(storage, entities);
            } else {
                restore_deleted(storage, entities);
            }
            let reselect = if forward {
                None
            } else {
                entities.first().copied()
            };
            select_entity(stage, storage, reselect);
            let viewer = &mut *stage.viewer;
            build_rows(viewer, storage);
            stage.mesh_state.full_rebuild_needed = true;
        }
        Edit::Material {
            name,
            before,
            after,
        } => {
            let value = if forward { after } else { before };
            reset_to(stage.materials, name, value);
            let viewer = &mut *stage.viewer;
            if viewer.selection.material.as_deref() == Some(name.as_str()) {
                viewer.selection.material_values = value.clone();
                viewer.view.detail_dirty = true;
            }
        }
        Edit::Light {
            entity,
            before,
            after,
        } => {
            let value = if forward { after } else { before };
            if let Some(light) = storage.get_mut::<Light>(*entity) {
                write_light(light, value);
            }
            let viewer = &mut *stage.viewer;
            if viewer.selection.selected == Some(*entity) {
                viewer.selection.light = Some(value.clone());
                viewer.view.detail_dirty = true;
            }
        }
        Edit::Camera {
            entity,
            before,
            after,
        } => {
            let value = if forward { after } else { before };
            if let Some(camera) = storage.get_mut::<Camera>(*entity) {
                write_camera(camera, value);
            }
            let viewer = &mut *stage.viewer;
            if viewer.selection.selected == Some(*entity) {
                viewer.selection.camera = Some(value.clone());
                viewer.view.detail_dirty = true;
            }
        }
    }
}
