use nightshade::prelude::*;
use nightshade_render::resources::outline::Outline;
use nightshade_scene::contract::LocalTransform;

use crate::commands::selection::{read_transform, select};
use crate::resources::ViewerResources;
use crate::theme::OUTLINE_COLOR;

pub fn refresh_selection(
    mut viewer: ResMut<ViewerResources>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let Some(entity) = viewer.selection.selected else {
        return;
    };
    if !storage.is_alive(entity) {
        select(viewer, storage, None);
        return;
    }
    let Some(transform) = storage.get::<LocalTransform>(entity) else {
        return;
    };
    let translation = [
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
    ];
    let scale = [transform.scale.x, transform.scale.y, transform.scale.z];
    let changed = translation != viewer.selection.translation
        || scale != viewer.selection.scale
        || transform.rotation != viewer.selection.rotation_quat;
    if changed {
        read_transform(&mut viewer.selection, transform);
        viewer.view.inspector_dirty = true;
    }
}

pub fn sync_outline(viewer: Res<ViewerResources>, mut outline: ResMut<Outline>) {
    outline.active_entity = viewer.selection.selected;
    outline.color = OUTLINE_COLOR;
}
