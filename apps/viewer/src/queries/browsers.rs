use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::data::{
    BrowserItem, BrowserTab, GltfFile, KHRONOS_BASE, KhronosRaw, ModelFiles, PAGE_SIZE,
    POLYHAVEN_THUMB, PolyRaw, SkyFiles,
};
use crate::queries::library::preview_key;
use crate::resources::{
    AssetSlot, BrowserEntry, FetchState, KhronosAsset, LoadingSlot, NoticeSlot, PolyAsset,
    ViewerResources,
};

pub fn browser_tab_index(tab: BrowserTab) -> usize {
    match tab {
        BrowserTab::Library => 0,
        BrowserTab::Khronos => 1,
        BrowserTab::Models => 2,
        BrowserTab::Skies => 3,
    }
}

pub fn index_ready<T>(state: &Arc<Mutex<FetchState<T>>>) -> bool {
    state
        .lock()
        .map(|guard| guard.loaded().is_some())
        .unwrap_or(false)
}

fn matches(name: &str, query: &str) -> bool {
    query.is_empty() || name.to_lowercase().contains(query)
}

pub fn page_count(viewer: &ViewerResources, tab: BrowserTab, query: &str) -> usize {
    matching_count(viewer, tab, query)
        .div_ceil(PAGE_SIZE)
        .max(1)
}

fn matching_count(viewer: &ViewerResources, tab: BrowserTab, query: &str) -> usize {
    match tab {
        BrowserTab::Library => viewer
            .library
            .entries
            .iter()
            .filter(|entry| matches(&entry.name, query))
            .count(),
        BrowserTab::Khronos => viewer
            .browsers
            .khronos
            .lock()
            .ok()
            .and_then(|guard| {
                guard.loaded().map(|entries| {
                    entries
                        .iter()
                        .filter(|entry| matches(&entry.label, query))
                        .count()
                })
            })
            .unwrap_or(0),
        BrowserTab::Models => poly_count(&viewer.browsers.models, query),
        BrowserTab::Skies => poly_count(&viewer.browsers.skies, query),
    }
}

fn poly_count(state: &Arc<Mutex<FetchState<Vec<PolyAsset>>>>, query: &str) -> usize {
    state
        .lock()
        .ok()
        .and_then(|guard| {
            guard.loaded().map(|entries| {
                entries
                    .iter()
                    .filter(|entry| matches(&entry.name, query))
                    .count()
            })
        })
        .unwrap_or(0)
}

pub fn page_entries(
    viewer: &ViewerResources,
    tab: BrowserTab,
    query: &str,
    page: usize,
) -> Vec<BrowserEntry> {
    let skip = page * PAGE_SIZE;
    match tab {
        BrowserTab::Library => viewer
            .library
            .entries
            .iter()
            .filter(|entry| matches(&entry.name, query))
            .skip(skip)
            .take(PAGE_SIZE)
            .map(|entry| BrowserEntry {
                label: entry.name.clone(),
                key: entry
                    .thumbnail
                    .clone()
                    .unwrap_or_else(|| preview_key(&entry.path)),
                item: BrowserItem::File(entry.path.clone()),
            })
            .collect(),
        BrowserTab::Khronos => {
            let Ok(guard) = viewer.browsers.khronos.lock() else {
                return Vec::new();
            };
            let Some(entries) = guard.loaded() else {
                return Vec::new();
            };
            entries
                .iter()
                .filter(|entry| matches(&entry.label, query))
                .skip(skip)
                .take(PAGE_SIZE)
                .map(|entry| BrowserEntry {
                    label: entry.label.clone(),
                    key: entry.thumbnail.clone().unwrap_or_default(),
                    item: BrowserItem::Khronos(entry.name.clone()),
                })
                .collect()
        }
        BrowserTab::Models => poly_entries(&viewer.browsers.models, query, skip, true),
        BrowserTab::Skies => poly_entries(&viewer.browsers.skies, query, skip, false),
    }
}

fn poly_entries(
    state: &Arc<Mutex<FetchState<Vec<PolyAsset>>>>,
    query: &str,
    skip: usize,
    models: bool,
) -> Vec<BrowserEntry> {
    let Ok(guard) = state.lock() else {
        return Vec::new();
    };
    let Some(entries) = guard.loaded() else {
        return Vec::new();
    };
    entries
        .iter()
        .filter(|entry| matches(&entry.name, query))
        .skip(skip)
        .take(PAGE_SIZE)
        .map(|entry| BrowserEntry {
            label: entry.name.clone(),
            key: entry.thumbnail.clone(),
            item: if models {
                BrowserItem::Model(entry.slug.clone())
            } else {
                BrowserItem::Sky(entry.slug.clone())
            },
        })
        .collect()
}

pub fn random_slug(state: &Arc<Mutex<FetchState<Vec<PolyAsset>>>>) -> Option<String> {
    let guard = state.lock().ok()?;
    let entries = guard.loaded()?;
    if entries.is_empty() {
        return None;
    }
    Some(entries[fastrand::usize(..entries.len())].slug.clone())
}

pub fn named_files(gltf: &[u8]) -> Vec<String> {
    let Ok(document) = serde_json::from_slice::<serde_json::Value>(gltf) else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for list in ["buffers", "images"] {
        let Some(entries) = document.get(list).and_then(|value| value.as_array()) else {
            continue;
        };
        for entry in entries {
            let Some(uri) = entry.get("uri").and_then(|value| value.as_str()) else {
                continue;
            };
            if uri.starts_with("data:") || uris.iter().any(|known| known == uri) {
                continue;
            }
            uris.push(uri.to_string());
        }
    }
    uris
}

pub fn arrived(result: ehttp::Result<ehttp::Response>) -> Option<ehttp::Response> {
    result.ok().filter(|response| response.ok)
}

pub fn khronos_entries(raw: Vec<KhronosRaw>) -> Vec<KhronosAsset> {
    let mut entries: Vec<KhronosAsset> = raw
        .into_iter()
        .filter_map(|entry| {
            let (model_url, binary) = match (
                entry.variants.get("glTF-Binary"),
                entry.variants.get("glTF"),
            ) {
                (Some(file), _) => (
                    format!("{KHRONOS_BASE}/{}/glTF-Binary/{file}", entry.name),
                    true,
                ),
                (None, Some(file)) => (format!("{KHRONOS_BASE}/{}/glTF/{file}", entry.name), false),
                (None, None) => return None,
            };
            let thumbnail = entry
                .screenshot
                .as_ref()
                .map(|path| format!("{KHRONOS_BASE}/{}/{path}", entry.name));
            Some(KhronosAsset {
                name: entry.name,
                label: entry.label,
                model_url,
                binary,
                thumbnail,
            })
        })
        .collect();
    entries.sort_by_key(|entry| entry.label.to_lowercase());
    entries
}

pub fn poly_entries_of(raw: BTreeMap<String, PolyRaw>) -> Vec<PolyAsset> {
    let mut entries: Vec<PolyAsset> = raw
        .into_iter()
        .map(|(slug, asset)| PolyAsset {
            thumbnail: format!("{POLYHAVEN_THUMB}{slug}.png?width=256&height=256"),
            slug,
            name: asset.name,
        })
        .collect();
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    entries
}

pub fn pick_sky(files: SkyFiles, preferred: u32) -> Option<String> {
    let entries: Vec<(u32, String)> = files
        .hdri
        .into_iter()
        .filter_map(|(key, resolution)| {
            resolution
                .hdr
                .map(|link| (resolution_value(&key), link.url))
        })
        .collect();
    pick_resolution(entries, preferred)
}

pub fn pick_model(files: ModelFiles, preferred: u32) -> Option<GltfFile> {
    let entries: Vec<(u32, GltfFile)> = files
        .gltf
        .into_iter()
        .map(|(key, resolution)| (resolution_value(&key), resolution.gltf))
        .collect();
    pick_resolution(entries, preferred)
}

fn pick_resolution<T>(mut entries: Vec<(u32, T)>, preferred: u32) -> Option<T> {
    entries.sort_by_key(|(value, _)| *value);
    if let Some(index) = entries.iter().position(|(value, _)| *value == preferred) {
        return Some(entries.swap_remove(index).1);
    }
    if let Some(index) = entries.iter().rposition(|(value, _)| *value <= preferred) {
        return Some(entries.swap_remove(index).1);
    }
    entries.into_iter().next().map(|(_, value)| value)
}

fn resolution_value(key: &str) -> u32 {
    key.trim_end_matches(['k', 'K']).parse().unwrap_or(u32::MAX)
}

pub fn empty_text(viewer: &ViewerResources, tab: BrowserTab) -> &'static str {
    let state = match tab {
        BrowserTab::Library => return library_text(viewer),
        BrowserTab::Khronos => index_state(&viewer.browsers.khronos),
        BrowserTab::Models => index_state(&viewer.browsers.models),
        BrowserTab::Skies => index_state(&viewer.browsers.skies),
    };
    match state {
        IndexState::Working => "Reading the library index.",
        IndexState::Failed => "The library did not answer. Use Refresh to ask again.",
        IndexState::Ready if !viewer.view.search.is_empty() => "Nothing matches that search.",
        IndexState::Ready => "The library holds nothing here.",
    }
}

fn library_text(viewer: &ViewerResources) -> &'static str {
    if viewer.library.folders.is_empty() && viewer.library.files.is_empty() {
        return "Add a folder and the assets in it stay here.";
    }
    if viewer.library.scanning {
        return "Reading those folders.";
    }
    if !viewer.view.search.is_empty() {
        return "Nothing matches that search.";
    }
    "Those folders hold no model and no sky."
}

enum IndexState {
    Working,
    Ready,
    Failed,
}

fn index_state<T>(state: &std::sync::Arc<std::sync::Mutex<FetchState<T>>>) -> IndexState {
    let Ok(guard) = state.lock() else {
        return IndexState::Working;
    };
    match &*guard {
        FetchState::Loaded(_) => IndexState::Ready,
        FetchState::Failed => IndexState::Failed,
        _ => IndexState::Working,
    }
}

pub fn slots(viewer: &ViewerResources) -> (AssetSlot, LoadingSlot, NoticeSlot) {
    (
        Arc::clone(&viewer.incoming.asset),
        Arc::clone(&viewer.incoming.loading),
        Arc::clone(&viewer.incoming.notice),
    )
}
