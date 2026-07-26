use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Texture,
    Material,
}

struct WatchedAsset {
    name: String,
    file_path: PathBuf,
    kind: AssetKind,
}

#[derive(Default)]
pub struct AssetWatcher {
    watched_assets: Vec<WatchedAsset>,
    key_to_index: HashMap<String, usize>,
    pending_watches: Vec<(String, PathBuf, AssetKind)>,
}

impl AssetWatcher {
    pub fn track_texture(&mut self, name: String, path: PathBuf) {
        self.pending_watches.push((name, path, AssetKind::Texture));
    }

    pub fn track_material(&mut self, name: String, path: PathBuf) {
        self.pending_watches.push((name, path, AssetKind::Material));
    }

    pub(crate) fn take_pending_watches(&mut self) -> Vec<(String, PathBuf, AssetKind)> {
        std::mem::take(&mut self.pending_watches)
    }

    pub(crate) fn register_asset(
        &mut self,
        key: String,
        name: String,
        path: PathBuf,
        kind: AssetKind,
    ) {
        let index = self.watched_assets.len();
        self.watched_assets.push(WatchedAsset {
            name,
            file_path: path,
            kind,
        });
        self.key_to_index.insert(key, index);
    }

    pub(crate) fn watched_keys(&self) -> Vec<String> {
        self.key_to_index.keys().cloned().collect()
    }

    pub(crate) fn get_by_key(&self, key: &str) -> Option<(AssetKind, String, PathBuf)> {
        self.key_to_index.get(key).map(|&index| {
            let watched = &self.watched_assets[index];
            (
                watched.kind,
                watched.name.clone(),
                watched.file_path.clone(),
            )
        })
    }
}
