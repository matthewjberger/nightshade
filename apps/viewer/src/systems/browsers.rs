use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::{ui_release_image_layer, ui_upload_image};

use crate::commands::browsers::{
    ensure_khronos, ensure_polyhaven, prune_thumbnails, request_thumbnails,
};
use crate::commands::library::{request_previews, start_scan, take_scan};
use crate::data::{
    BrowserTab, LIBRARY_IDLE_FRAMES, LIBRARY_WATCH_FRAMES, MAX_THUMBNAILS, POLYHAVEN_MODELS,
    POLYHAVEN_SKIES,
};
use crate::queries::browsers::{browser_tab_index, index_ready};
use crate::queries::library::is_remote;
use crate::queries::library::roots;
use crate::resources::{ThumbnailSlot, ViewerResources};

pub fn ensure_indices(mut viewer: ResMut<ViewerResources>) {
    if viewer.browsers.started {
        return;
    }
    viewer.browsers.started = true;
    ensure_khronos(&viewer.browsers.khronos);
    ensure_polyhaven(&viewer.browsers.skies, POLYHAVEN_SKIES);
    ensure_polyhaven(&viewer.browsers.models, POLYHAVEN_MODELS);
}

pub fn watch_library(mut viewer: ResMut<ViewerResources>) {
    take_scan(&mut viewer);
    if viewer.library.scanning {
        return;
    }
    if viewer.library.watch > 0 {
        viewer.library.watch -= 1;
        return;
    }
    if roots(&viewer).is_empty() && viewer.library.files.is_empty() {
        return;
    }
    start_scan(&mut viewer);
    viewer.library.watch = if viewer.view.browser == Some(BrowserTab::Library) {
        LIBRARY_WATCH_FRAMES
    } else {
        LIBRARY_IDLE_FRAMES
    };
}

pub fn poll_browsers(mut viewer: ResMut<ViewerResources>, mut build: UiBuild<'_>) {
    let viewer = &mut *viewer;

    for (index, ready) in [
        index_ready(&viewer.browsers.khronos),
        index_ready(&viewer.browsers.models),
        index_ready(&viewer.browsers.skies),
    ]
    .into_iter()
    .enumerate()
    {
        if ready && !viewer.browsers.announced[index] {
            viewer.browsers.announced[index] = true;
            if viewer.view.browser.map(browser_tab_index) == Some(index) {
                viewer.view.browser_dirty = true;
            }
        }
    }

    if std::mem::take(&mut viewer.browsers.reset_thumbnails) {
        let context = build.context();
        for slot in viewer.browsers.slots.values() {
            ui_release_image_layer(context.images, slot.layer);
        }
        viewer.browsers.slots.clear();
        viewer.browsers.requested.clear();
    }

    if viewer.view.browser.is_some() {
        let wanted = std::mem::take(&mut viewer.browsers.wanted);
        prune_thumbnails(viewer, &mut build, &wanted);
        let (remote, local): (Vec<String>, Vec<String>) =
            wanted.iter().cloned().partition(|key| is_remote(key));
        request_thumbnails(viewer, &remote);
        request_previews(viewer, &local);
        viewer.browsers.wanted = wanted;
    }

    let mut decoded = viewer
        .browsers
        .decoded
        .lock()
        .ok()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default();
    if decoded.is_empty() {
        return;
    }
    let context = build.context();
    let mut added = false;
    for thumbnail in decoded.drain(..) {
        let known = viewer.browsers.slots.contains_key(&thumbnail.key);
        if !known && viewer.browsers.slots.len() >= MAX_THUMBNAILS {
            continue;
        }
        if let Some(upload) = ui_upload_image(
            context.images,
            context.runtime,
            &thumbnail.pixels,
            thumbnail.width,
            thumbnail.height,
        ) {
            let previous = viewer.browsers.slots.insert(
                thumbnail.key,
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
    }
    if added && viewer.view.browser.is_some() {
        viewer.view.browser_dirty = true;
    }
}
