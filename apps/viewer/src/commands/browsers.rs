use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::ui_release_image_layer;

use crate::commands::notice::report;
use crate::data::{
    KHRONOS_INDEX, KhronosRaw, MAX_THUMBNAILS, ModelFiles, POLYHAVEN_FILES, PolyRaw, SkyFiles,
};
use crate::queries::browsers::{
    arrived, khronos_entries, named_files, pick_model, pick_sky, poly_entries_of, slots,
};
use crate::resources::{
    AssetSlot, FetchState, KhronosAsset, LoadingSlot, NoticeSlot, PendingAsset, PolyAsset,
    ThumbnailImage, ViewerResources,
};
use crate::theme::THUMBNAIL_SIZE;

pub fn refresh_indices(viewer: &mut ViewerResources) {
    reset_index(&viewer.browsers.khronos);
    reset_index(&viewer.browsers.models);
    reset_index(&viewer.browsers.skies);
    viewer.browsers.started = false;
    viewer.browsers.announced = [false; 3];
    viewer.browsers.reset_thumbnails = true;
    viewer.view.browser_dirty = true;
}

fn reset_index<T>(state: &Arc<Mutex<FetchState<T>>>) {
    if let Ok(mut guard) = state.lock() {
        *guard = FetchState::Idle;
    }
}

const THUMBNAILS_PER_FRAME: usize = 4;

type DecodedSlot = Arc<Mutex<Vec<ThumbnailImage>>>;

pub fn prune_thumbnails(viewer: &mut ViewerResources, build: &mut UiBuild<'_>, wanted: &[String]) {
    if viewer.browsers.slots.len() + wanted.len() <= MAX_THUMBNAILS {
        return;
    }
    let keep: HashSet<&String> = wanted.iter().collect();
    let context = build.context();
    let stale: Vec<String> = viewer
        .browsers
        .slots
        .keys()
        .filter(|key| !keep.contains(key))
        .cloned()
        .collect();
    for key in stale {
        if let Some(slot) = viewer.browsers.slots.remove(&key) {
            ui_release_image_layer(context.images, slot.layer);
        }
        viewer.browsers.requested.remove(&key);
    }
}

pub fn request_thumbnails(viewer: &mut ViewerResources, wanted: &[String]) {
    let mut started = 0;
    for url in wanted {
        if started >= THUMBNAILS_PER_FRAME || viewer.browsers.slots.len() >= MAX_THUMBNAILS {
            break;
        }
        if viewer.browsers.slots.contains_key(url) || viewer.browsers.requested.contains(url) {
            continue;
        }
        viewer.browsers.requested.insert(url.clone());
        fetch_thumbnail(Arc::clone(&viewer.browsers.decoded), url.clone());
        started += 1;
    }
}

fn fetch_thumbnail(decoded: DecodedSlot, url: String) {
    let key = url.clone();
    ehttp::fetch(ehttp::Request::get(&url), move |result| {
        let Ok(response) = result else {
            return;
        };
        if !response.ok {
            return;
        }
        let Ok(image) = image::load_from_memory(&response.bytes) else {
            return;
        };
        let thumbnail = image.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE).to_rgba8();
        let (width, height) = thumbnail.dimensions();
        if let Ok(mut queue) = decoded.lock() {
            queue.push(ThumbnailImage {
                key,
                pixels: thumbnail.into_raw(),
                width,
                height,
            });
        }
    });
}

pub fn fetch_khronos(viewer: &ViewerResources, name: &str) {
    let (url, binary) = {
        let Ok(guard) = viewer.browsers.khronos.lock() else {
            return;
        };
        let Some(entries) = guard.loaded() else {
            return;
        };
        match entries.iter().find(|entry| entry.name == name) {
            Some(entry) => (entry.model_url.clone(), entry.binary),
            None => return,
        }
    };
    if !begin_loading(viewer, name) {
        return;
    }
    let (asset, loading, notice) = slots(viewer);
    if binary {
        download_single(url, PendingAsset::Model, asset, loading, notice);
        return;
    }
    let base = url
        .rsplit_once('/')
        .map(|(above, _)| format!("{above}/"))
        .unwrap_or_default();
    ehttp::fetch(ehttp::Request::get(&url), move |result| {
        let Some(gltf) = arrived(result).map(|response| response.bytes) else {
            fail(&loading, &notice, "The model file did not download.");
            return;
        };
        let includes: Vec<(String, String)> = named_files(&gltf)
            .into_iter()
            .map(|uri| {
                let key = percent_encoding::percent_decode_str(&uri)
                    .decode_utf8_lossy()
                    .to_string();
                (key, format!("{base}{uri}"))
            })
            .collect();
        download_parts(gltf, includes, asset, loading, notice);
    });
}

pub fn fetch_polyhaven_sky(viewer: &ViewerResources, slug: &str, resolution: u32) {
    if !begin_loading(viewer, slug) {
        return;
    }
    let (asset, loading, notice) = slots(viewer);
    let files_url = format!("{POLYHAVEN_FILES}{slug}");
    ehttp::fetch(ehttp::Request::get(&files_url), move |result| {
        let url = arrived(result)
            .and_then(|response| serde_json::from_slice::<SkyFiles>(&response.bytes).ok())
            .and_then(|files| pick_sky(files, resolution));
        match url {
            Some(url) => download_single(url, PendingAsset::Sky, asset, loading, notice),
            None => fail(
                &loading,
                &notice,
                "That sky is not offered at any resolution the viewer can read.",
            ),
        }
    });
}

pub fn fetch_polyhaven_model(viewer: &ViewerResources, slug: &str, resolution: u32) {
    if !begin_loading(viewer, slug) {
        return;
    }
    let (asset, loading, notice) = slots(viewer);
    let files_url = format!("{POLYHAVEN_FILES}{slug}");
    ehttp::fetch(ehttp::Request::get(&files_url), move |result| {
        let gltf = arrived(result)
            .and_then(|response| serde_json::from_slice::<ModelFiles>(&response.bytes).ok())
            .and_then(|files| pick_model(files, resolution));
        match gltf {
            Some(gltf) => {
                let includes: Vec<(String, String)> = gltf
                    .include
                    .into_iter()
                    .map(|(key, file)| (key, file.url))
                    .collect();
                ehttp::fetch(
                    ehttp::Request::get(&gltf.url),
                    move |result| match arrived(result).map(|response| response.bytes) {
                        Some(bytes) => download_parts(bytes, includes, asset, loading, notice),
                        None => fail(&loading, &notice, "The model file did not download."),
                    },
                );
            }
            None => fail(
                &loading,
                &notice,
                "That model is not offered at any resolution the viewer can read.",
            ),
        }
    });
}

fn begin_loading(viewer: &ViewerResources, label: &str) -> bool {
    let Ok(mut guard) = viewer.incoming.loading.lock() else {
        return false;
    };
    if guard.is_some() {
        return false;
    }
    *guard = Some(label.to_string());
    true
}

fn fail(loading: &LoadingSlot, notice: &NoticeSlot, message: impl Into<String>) {
    if let Ok(mut guard) = loading.lock() {
        *guard = None;
    }
    report(notice, message);
}

fn download_single(
    url: String,
    make: fn(Vec<u8>) -> PendingAsset,
    asset: AssetSlot,
    loading: LoadingSlot,
    notice: NoticeSlot,
) {
    ehttp::fetch(ehttp::Request::get(&url), move |result| {
        if let Ok(response) = result
            && response.ok
            && let Ok(mut slot) = asset.lock()
        {
            *slot = Some(make(response.bytes));
        } else {
            fail(&loading, &notice, "The download did not finish.");
        }
    });
}

fn download_parts(
    gltf: Vec<u8>,
    includes: Vec<(String, String)>,
    asset: AssetSlot,
    loading: LoadingSlot,
    notice: NoticeSlot,
) {
    if includes.is_empty() {
        if let Ok(mut slot) = asset.lock() {
            *slot = Some(PendingAsset::ModelWithResources {
                gltf,
                resources: HashMap::new(),
            });
        }
        return;
    }

    let total = includes.len();
    let progress = Arc::new(Mutex::new((Some(gltf), HashMap::new(), 0usize, false)));
    for (key, url) in includes {
        let progress = Arc::clone(&progress);
        let asset = Arc::clone(&asset);
        let loading = Arc::clone(&loading);
        let notice = Arc::clone(&notice);
        ehttp::fetch(ehttp::Request::get(&url), move |result| {
            let bytes = arrived(result).map(|response| response.bytes);
            let Ok(mut guard) = progress.lock() else {
                return;
            };
            match bytes {
                Some(bytes) => {
                    guard.1.insert(key.clone(), bytes);
                    guard.2 += 1;
                }
                None => guard.3 = true,
            }
            if guard.3 {
                fail(
                    &loading,
                    &notice,
                    "One of the files the model names did not download.",
                );
                return;
            }
            if guard.2 == total
                && let Some(gltf) = guard.0.take()
            {
                let resources = std::mem::take(&mut guard.1);
                if let Ok(mut slot) = asset.lock() {
                    *slot = Some(PendingAsset::ModelWithResources { gltf, resources });
                }
            }
        });
    }
}

pub fn ensure_khronos(state: &Arc<Mutex<FetchState<Vec<KhronosAsset>>>>) {
    {
        let Ok(mut guard) = state.lock() else {
            return;
        };
        if !matches!(*guard, FetchState::Idle) {
            return;
        }
        *guard = FetchState::Loading;
    }
    let target = Arc::clone(state);
    ehttp::fetch(ehttp::Request::get(KHRONOS_INDEX), move |result| {
        let next = match result {
            Ok(response) if response.ok => {
                match serde_json::from_slice::<Vec<KhronosRaw>>(&response.bytes) {
                    Ok(raw) => FetchState::Loaded(khronos_entries(raw)),
                    Err(_) => FetchState::Failed,
                }
            }
            _ => FetchState::Failed,
        };
        if let Ok(mut guard) = target.lock() {
            *guard = next;
        }
    });
}

pub fn ensure_polyhaven(state: &Arc<Mutex<FetchState<Vec<PolyAsset>>>>, index_url: &str) {
    {
        let Ok(mut guard) = state.lock() else {
            return;
        };
        if !matches!(*guard, FetchState::Idle) {
            return;
        }
        *guard = FetchState::Loading;
    }
    let target = Arc::clone(state);
    ehttp::fetch(ehttp::Request::get(index_url), move |result| {
        let next = match result {
            Ok(response) if response.ok => {
                match serde_json::from_slice::<BTreeMap<String, PolyRaw>>(&response.bytes) {
                    Ok(raw) => FetchState::Loaded(poly_entries_of(raw)),
                    Err(_) => FetchState::Failed,
                }
            }
            _ => FetchState::Failed,
        };
        if let Ok(mut guard) = target.lock() {
            *guard = next;
        }
    });
}
