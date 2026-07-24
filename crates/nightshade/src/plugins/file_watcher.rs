use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;

pub struct FileWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    receiver: std::sync::Mutex<mpsc::Receiver<Result<Event, notify::Error>>>,
    sender: mpsc::Sender<Result<Event, notify::Error>>,
    path_to_key: HashMap<PathBuf, String>,
    key_to_path: HashMap<String, PathBuf>,
    changed_keys: HashSet<String>,
}

impl Default for FileWatcher {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        let watcher_sender = sender.clone();
        let watcher = notify::recommended_watcher(watcher_sender).ok();
        Self {
            watcher,
            receiver: std::sync::Mutex::new(receiver),
            sender,
            path_to_key: HashMap::new(),
            key_to_path: HashMap::new(),
            changed_keys: HashSet::new(),
        }
    }
}

impl FileWatcher {
    pub fn watch(&mut self, key: String, path: PathBuf) {
        if self.key_to_path.contains_key(&key) {
            return;
        }
        let canonical = path.canonicalize().unwrap_or(path);
        tracing::info!("Watching file [{}]: {}", key, canonical.display());
        self.path_to_key.insert(canonical.clone(), key.clone());
        self.key_to_path.insert(key, canonical.clone());
        if let Some(watcher) = &mut self.watcher {
            let _ = watcher.watch(&canonical, RecursiveMode::NonRecursive);
        }
    }

    pub fn poll(&mut self) {
        let mut events = Vec::new();
        while let Ok(event_result) = self.receiver.lock().unwrap().try_recv() {
            if let Ok(event) = event_result {
                events.push(event);
            }
        }
        for event in events {
            if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                continue;
            }
            for path in event.paths {
                let canonical = path.canonicalize().unwrap_or(path);
                if let Some(key) = self.path_to_key.get(&canonical) {
                    self.changed_keys.insert(key.clone());
                }
            }
        }

        if self.watcher.is_none() {
            let watcher_sender = self.sender.clone();
            if let Ok(mut new_watcher) = notify::recommended_watcher(watcher_sender) {
                for path in self.key_to_path.values() {
                    let _ = new_watcher.watch(path, RecursiveMode::NonRecursive);
                }
                self.watcher = Some(new_watcher);
            }
        }
    }

    pub fn take_change(&mut self, key: &str) -> bool {
        self.changed_keys.remove(key)
    }
}

pub fn poll_file_watcher_system(world: &mut crate::ecs::world::World) {
    world.plugin_resource_mut::<FileWatcher>().poll();
}

/// Installs hot-reload file watching: the file and asset watcher polls run
/// at the end of the frame, alongside the other housekeeping passes. Watch
/// a file with `world.plugin_resource_mut::<FileWatcher>().watch(key, path)`
/// and consume changes with `take_change(key)`.
pub struct FileWatcherPlugin;

impl crate::app::Plugin for FileWatcherPlugin {
    fn build(&self, app: &mut crate::app::App) {
        use crate::app::Stage;
        app.world
            .insert_resource(crate::plugins::file_watcher::FileWatcher::default());
        app.add_system(Stage::Last, poll_file_watcher_system);
        #[cfg(feature = "assets")]
        app.add_system(Stage::Last, poll_asset_watcher_system);
    }
}

use crate::ecs::asset_watcher::{AssetKind, AssetWatcher};
use crate::ecs::world::commands::{EcsCommand, RenderCommand};
use crate::prelude::{queue_ecs_command, queue_render_command};

/// Applies pending asset watches to the OS file watcher and reloads any
/// texture or material whose backing file changed on disk.
pub fn poll_asset_watcher_system(world: &mut crate::ecs::world::World) {
    if world.ecs.resource::<FileWatcher>().is_none() {
        return;
    }
    let pending = world
        .plugin_resource_mut::<AssetWatcher>()
        .take_pending_watches();
    for (name, path, kind) in pending {
        let key = format!("asset:{name}");
        world
            .plugin_resource_mut::<FileWatcher>()
            .watch(key.clone(), path.clone());
        world
            .plugin_resource_mut::<AssetWatcher>()
            .register_asset(key, name, path, kind);
    }

    let keys = world.plugin_resource::<AssetWatcher>().watched_keys();
    for key in keys {
        if world.plugin_resource_mut::<FileWatcher>().take_change(&key)
            && let Some((kind, name, path)) =
                world.plugin_resource::<AssetWatcher>().get_by_key(&key)
        {
            match kind {
                AssetKind::Texture => reload_texture(world, name, path),
                AssetKind::Material => reload_material(world, name, path),
            }
        }
    }
}

fn reload_texture(world: &mut crate::ecs::world::World, name: String, path: PathBuf) {
    tracing::info!(
        "File changed on disk, reloading texture: {}",
        path.display()
    );
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!("Failed to read changed file {}: {}", path.display(), error);
            return;
        }
    };
    let image = match image::load_from_memory(&bytes) {
        Ok(image) => image.to_rgba8(),
        Err(error) => {
            tracing::warn!(
                "Failed to decode changed image {}: {}",
                path.display(),
                error
            );
            return;
        }
    };
    let (width, height) = image.dimensions();
    tracing::info!("Queuing texture reload: {} ({}x{})", name, width, height);
    queue_render_command(
        world,
        RenderCommand::ReloadTexture {
            name,
            rgba_data: image.into_raw(),
            width,
            height,
        },
    );
}

fn reload_material(world: &mut crate::ecs::world::World, name: String, path: PathBuf) {
    tracing::info!(
        "File changed on disk, reloading material: {}",
        path.display()
    );
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            tracing::warn!("Failed to read material file {}: {}", path.display(), error);
            return;
        }
    };
    let material: crate::render::material::Material = match serde_json::from_str(&contents) {
        Ok(material) => material,
        Err(error) => {
            tracing::warn!(
                "Failed to parse material JSON {}: {}",
                path.display(),
                error
            );
            return;
        }
    };
    tracing::info!("Queuing material reload: {}", name);
    queue_ecs_command(
        world,
        EcsCommand::ReloadMaterial {
            name,
            material: Box::new(material),
        },
    );
}
