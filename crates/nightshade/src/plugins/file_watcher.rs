use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;

pub struct FileWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    receiver: mpsc::Receiver<Result<Event, notify::Error>>,
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
            receiver,
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
        while let Ok(event_result) = self.receiver.try_recv() {
            let Ok(event) = event_result else {
                continue;
            };
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
    world.resources.file_watcher.poll();
}

/// Registers the file and asset watcher polls at the end of the frame,
/// alongside the other housekeeping passes.
pub fn install(world: &mut crate::ecs::world::World) {
    use crate::schedule::{FramePhase, schedule_push};
    let schedule = &mut world.resources.schedules.frame;
    schedule_push(schedule, FramePhase::Last, poll_file_watcher_system);
    #[cfg(feature = "assets")]
    schedule_push(
        schedule,
        FramePhase::Last,
        crate::ecs::asset_watcher::poll_asset_watcher_system,
    );
}

/// Installs hot-reload file watching through [`install`]. Watch a file
/// with `world.resources.file_watcher.watch(key, path)` and consume
/// changes with `take_change(key)`.
pub struct FileWatcherPlugin;

impl crate::app::Plugin for FileWatcherPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.add_startup_system(install);
    }
}
