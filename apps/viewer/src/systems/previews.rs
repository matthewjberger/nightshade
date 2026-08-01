use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::{ui_release_image_layer, ui_upload_image};

use crate::resources::{ThumbnailSlot, ViewerResources};

pub fn upload_previews(mut viewer: ResMut<ViewerResources>, mut build: UiBuild<'_>) {
    let viewer = &mut *viewer;

    if std::mem::take(&mut viewer.model.previews_stale) {
        let context = build.context();
        for slot in viewer.model.previews.values() {
            ui_release_image_layer(context.images, slot.layer);
        }
        viewer.model.previews.clear();
    }

    let decoded = viewer
        .model
        .previews_decoded
        .lock()
        .ok()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default();
    if decoded.is_empty() {
        return;
    }
    let context = build.context();
    let mut added = false;
    for preview in decoded {
        let Some(upload) = ui_upload_image(
            context.images,
            context.runtime,
            &preview.pixels,
            preview.width,
            preview.height,
        ) else {
            continue;
        };
        let previous = viewer.model.previews.insert(
            preview.key,
            ThumbnailSlot {
                layer: upload.layer,
                uv_min: upload.uv_min,
                uv_max: upload.uv_max,
            },
        );
        if let Some(previous) = previous {
            ui_release_image_layer(context.images, previous.layer);
        }
        added = true;
    }
    if added && viewer.selection.material.is_some() {
        viewer.view.detail_dirty = true;
    }
}
