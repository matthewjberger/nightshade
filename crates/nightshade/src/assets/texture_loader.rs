//! Asynchronous texture loading with platform-specific implementations.
//!
//! Load textures in the background without blocking the main thread:
//!
//! - [`TextureLoadQueue`]: Queue for pending/in-flight/completed texture loads
//! - [`SharedTextureQueue`]: Thread-safe handle for async operations
//! - [`AssetLoadingState`]: Progress tracking for loading screens
//! - `process_texture_queue`: Process pending loads each frame
//! - `process_and_load_textures`: Combined process + upload to GPU
//!
//! # Platform Behavior
//!
//! | Platform | Loading Method |
//! |----------|----------------|
//! | Desktop | Synchronous file read from disk |
//! | WASM | Async HTTP fetch via ehttp |
//!
//! # Basic Usage
//!
//! ```ignore
//! // Create shared queue
//! let queue = create_shared_queue();
//!
//! // Queue textures for loading
//! queue_texture_from_path(&queue, "assets/textures/albedo.png");
//! queue_texture_from_path(&queue, "assets/textures/normal.png");
//!
//! // Track loading state
//! let mut loading_state = AssetLoadingState::new(2);  // 2 textures
//!
//! // Process each frame
//! fn run_systems(&mut self, world: &mut World) {
//!     let status = process_and_load_textures(
//!         &self.queue,
//!         world,
//!         &mut self.loading_state,
//!         4,  // Max concurrent loads
//!     );
//!
//!     if status == AssetLoadingStatus::Complete {
//!         println!("All textures loaded!");
//!     }
//! }
//! ```
//!
//! # Loading Screen Pattern
//!
//! ```ignore
//! struct LoadingScreen {
//!     queue: SharedTextureQueue,
//!     state: AssetLoadingState,
//! }
//!
//! impl LoadingScreen {
//!     fn new(texture_paths: &[&str]) -> Self {
//!         let queue = create_shared_queue();
//!         for path in texture_paths {
//!             queue_texture_from_path(&queue, path);
//!         }
//!         Self {
//!             queue,
//!             state: AssetLoadingState::new(texture_paths.len()),
//!         }
//!     }
//!
//!     fn update(&mut self, world: &mut World) -> bool {
//!         let status = process_and_load_textures(&self.queue, world, &mut self.state, 4);
//!         status == AssetLoadingStatus::Complete
//!     }
//!
//!     fn progress(&self) -> f32 {
//!         self.state.progress()  // 0.0 to 1.0
//!     }
//! }
//! ```
//!
//! # Asset Search Paths
//!
//! Configure where to search for texture files:
//!
//! ```ignore
//! // Set search paths at startup
//! set_asset_search_paths(vec![
//!     "assets/".to_string(),
//!     "content/textures/".to_string(),
//! ]);
//!
//! // Now queue_texture_from_path will search these directories
//! queue_texture_from_path(&queue, "player.png");
//! // Searches: assets/player.png, content/textures/player.png
//! ```
//!
//! # Manual Queue Control
//!
//! ```ignore
//! // Queue with explicit URL (WASM) or path (desktop)
//! if let Ok(mut q) = queue.lock() {
//!     q.queue_texture("my_texture".to_string(), "https://example.com/tex.png".to_string());
//! }
//!
//! // Process without auto-upload
//! process_texture_queue(&queue, 4);
//!
//! // Manually handle completed textures
//! if let Ok(mut q) = queue.lock() {
//!     for loaded in q.take_completed() {
//!         crate::assets::loading::queue_decoded_texture(
//!             world,
//!             loaded.name,
//!             loaded.rgba_data,
//!             loaded.width,
//!             loaded.height,
//!             TextureUsage::Color,
//!             SamplerSettings::DEFAULT,
//!         );
//!     }
//! }
//! ```
//!
//! # AssetLoadingState
//!
//! | Method | Description |
//! |--------|-------------|
//! | `progress()` | Returns 0.0-1.0 completion ratio |
//! | `is_complete()` | True when all textures processed |
//! | `loaded_textures` | Count of successfully loaded |
//! | `failed_textures` | Count of failed loads |
//!
//! # Error Handling
//!
//! Failed loads are counted but don't block completion. Check `failed_textures`
//! after loading completes to detect missing assets.

use crate::ecs::world::World;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

static ASSET_SEARCH_PATHS: OnceLock<Vec<String>> = OnceLock::new();

pub fn set_asset_search_paths(paths: Vec<String>) {
    let _ = ASSET_SEARCH_PATHS.set(paths);
}

pub fn get_asset_search_paths() -> &'static [String] {
    ASSET_SEARCH_PATHS.get().map_or(&[], |v| v.as_slice())
}

#[derive(Default)]
pub struct TextureLoadQueue {
    pending: VecDeque<TextureLoadRequest>,
    in_flight: Vec<String>,
    completed: Vec<LoadedTexture>,
    failed: usize,
}

pub struct TextureLoadRequest {
    pub url: String,
    pub name: String,
}

pub struct LoadedTexture {
    pub name: String,
    pub rgba_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl TextureLoadQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_texture(&mut self, name: String, url: String) {
        if !self.in_flight.contains(&name)
            && !self.pending.iter().any(|r| r.name == name)
            && !self.completed.iter().any(|t| t.name == name)
        {
            self.pending.push_back(TextureLoadRequest { url, name });
        }
    }

    pub fn take_completed(&mut self) -> Vec<LoadedTexture> {
        std::mem::take(&mut self.completed)
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty() || !self.in_flight.is_empty()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len() + self.in_flight.len()
    }

    pub fn failed_count(&self) -> usize {
        self.failed
    }

    pub fn processed_count(&self) -> usize {
        self.completed.len() + self.failed
    }
}

pub type SharedTextureQueue = Arc<Mutex<TextureLoadQueue>>;

pub fn create_shared_queue() -> SharedTextureQueue {
    Arc::new(Mutex::new(TextureLoadQueue::new()))
}

#[cfg(target_arch = "wasm32")]
pub fn process_texture_queue(queue: &SharedTextureQueue, max_concurrent: usize) {
    let requests: Vec<TextureLoadRequest> = {
        let mut q = queue.lock().unwrap();
        let available_slots = max_concurrent.saturating_sub(q.in_flight.len());
        let mut requests = Vec::new();
        for _ in 0..available_slots {
            if let Some(req) = q.pending.pop_front() {
                q.in_flight.push(req.name.clone());
                requests.push(req);
            }
        }
        requests
    };

    for request in requests {
        let queue_clone = Arc::clone(queue);
        let name = request.name.clone();

        ehttp::fetch(
            ehttp::Request::get(&request.url),
            move |result: ehttp::Result<ehttp::Response>| {
                let mut q = queue_clone.lock().unwrap();
                q.in_flight.retain(|n| n != &name);

                match result {
                    Ok(response) => {
                        if response.ok {
                            if let Ok(img) = image::load_from_memory(&response.bytes) {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                q.completed.push(LoadedTexture {
                                    name,
                                    rgba_data: rgba.into_raw(),
                                    width,
                                    height,
                                });
                            } else {
                                tracing::warn!("Failed to decode image: {}", name);
                                q.failed += 1;
                            }
                        } else {
                            tracing::warn!(
                                "HTTP error loading texture {}: {}",
                                name,
                                response.status
                            );
                            q.failed += 1;
                        }
                    }
                    Err(error) => {
                        tracing::warn!("Failed to fetch texture {}: {}", name, error);
                        q.failed += 1;
                    }
                }
            },
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn process_texture_queue(queue: &SharedTextureQueue, max_concurrent: usize) {
    use std::path::Path;

    let requests: Vec<TextureLoadRequest> = {
        let mut q = queue.lock().unwrap();
        let mut http_dispatched = Vec::new();
        let drained: Vec<TextureLoadRequest> = std::mem::take(&mut q.pending).into();
        let mut filesystem = Vec::new();
        for request in drained {
            if is_http_url(&request.url) {
                if q.in_flight.len() + http_dispatched.len() >= max_concurrent.max(1) {
                    q.pending.push_back(request);
                } else {
                    q.in_flight.push(request.name.clone());
                    http_dispatched.push(request);
                }
            } else {
                filesystem.push(request);
            }
        }
        for request in http_dispatched {
            dispatch_http_fetch(queue, request);
        }
        filesystem
    };

    for request in requests {
        let path = Path::new(&request.url);
        let bytes = if path.exists() {
            std::fs::read(path)
        } else {
            find_texture_file(&request.url)
        };

        match bytes {
            Ok(bytes) => {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    let mut q = queue.lock().unwrap();
                    q.completed.push(LoadedTexture {
                        name: request.name,
                        rgba_data: rgba.into_raw(),
                        width,
                        height,
                    });
                } else {
                    tracing::warn!("Failed to decode image: {}", request.name);
                    let mut q = queue.lock().unwrap();
                    q.failed += 1;
                }
            }
            Err(error) => {
                tracing::warn!("Failed to read texture {}: {}", request.name, error);
                let mut q = queue.lock().unwrap();
                q.failed += 1;
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_http_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[cfg(not(target_arch = "wasm32"))]
fn dispatch_http_fetch(queue: &SharedTextureQueue, request: TextureLoadRequest) {
    let queue_clone = Arc::clone(queue);
    let name = request.name.clone();
    ehttp::fetch(
        ehttp::Request::get(&request.url),
        move |result: ehttp::Result<ehttp::Response>| {
            let mut q = queue_clone.lock().unwrap();
            q.in_flight.retain(|n| n != &name);
            match result {
                Ok(response) if response.ok => {
                    if let Ok(img) = image::load_from_memory(&response.bytes) {
                        let rgba = img.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        q.completed.push(LoadedTexture {
                            name,
                            rgba_data: rgba.into_raw(),
                            width,
                            height,
                        });
                    } else {
                        tracing::warn!("Failed to decode image: {}", name);
                        q.failed += 1;
                    }
                }
                Ok(response) => {
                    tracing::warn!("HTTP {} for texture {}", response.status, name);
                    q.failed += 1;
                }
                Err(error) => {
                    tracing::warn!("Failed to fetch texture {}: {}", name, error);
                    q.failed += 1;
                }
            }
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn find_texture_file(url: &str) -> std::io::Result<Vec<u8>> {
    use std::path::Path;

    let candidate = Path::new(url);
    if candidate.exists() {
        return std::fs::read(candidate);
    }

    for prefix in get_asset_search_paths() {
        let candidate = Path::new(prefix).join(url);
        if candidate.exists() {
            return std::fs::read(&candidate);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("Texture not found: {}", url),
    ))
}

#[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
fn resolve_texture_path(name: &str) -> Option<std::path::PathBuf> {
    use std::path::Path;

    let candidate = Path::new(name);
    if candidate.exists() {
        return Some(candidate.to_path_buf());
    }
    for prefix in get_asset_search_paths() {
        let candidate = Path::new(prefix).join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn queue_texture_from_path(queue: &SharedTextureQueue, path: &str) {
    let normalized = path.replace('\\', "/");
    let name = normalized.clone();

    #[cfg(target_arch = "wasm32")]
    let url = {
        if let Some(stripped) = normalized.strip_prefix("assets/") {
            stripped.to_string()
        } else {
            normalized
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let url = normalized;

    if let Ok(mut q) = queue.lock() {
        q.queue_texture(name, url);
    }
}

#[derive(Default, Clone, Copy)]
pub struct AssetLoadingState {
    pub total_textures: usize,
    pub loaded_textures: usize,
    pub failed_textures: usize,
}

impl AssetLoadingState {
    pub fn new(total_textures: usize) -> Self {
        Self {
            total_textures,
            loaded_textures: 0,
            failed_textures: 0,
        }
    }

    pub fn progress(&self) -> f32 {
        if self.total_textures == 0 {
            return 1.0;
        }
        (self.loaded_textures + self.failed_textures) as f32 / self.total_textures as f32
    }

    pub fn is_complete(&self) -> bool {
        self.total_textures > 0
            && (self.loaded_textures + self.failed_textures) >= self.total_textures
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AssetLoadingStatus {
    #[default]
    Loading,
    Complete,
}

pub fn process_and_load_textures(
    queue: &SharedTextureQueue,
    world: &mut World,
    loading_state: &mut AssetLoadingState,
    max_concurrent: usize,
) -> AssetLoadingStatus {
    process_texture_queue(queue, max_concurrent);

    if let Ok(mut queue_guard) = queue.lock() {
        let completed = queue_guard.take_completed();
        let failed = queue_guard.failed_count();
        loading_state.loaded_textures += completed.len();
        loading_state.failed_textures = failed;

        for loaded in completed {
            #[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
            {
                let resolved_path = resolve_texture_path(&loaded.name);
                if let Some(path) = resolved_path {
                    world
                        .plugin_resource_mut::<crate::assets::asset_watcher::AssetWatcher>()
                        .track_texture(loaded.name.clone(), path);
                }
            }
            crate::assets::loading::queue_decoded_texture(
                world,
                loaded.name,
                loaded.rgba_data,
                loaded.width,
                loaded.height,
                crate::render::texture_data::TextureUsage::Color,
                crate::render::texture_data::SamplerSettings::DEFAULT,
            );
        }

        let pending = queue_guard.has_pending();
        let all_processed =
            (loading_state.loaded_textures + failed) >= loading_state.total_textures;

        if !pending && all_processed {
            if failed > 0 {
                tracing::warn!(
                    "Asset loading complete: {} textures loaded, {} failed",
                    loading_state.loaded_textures,
                    failed
                );
            } else if loading_state.loaded_textures > 0 {
                tracing::info!(
                    "Asset loading complete: {} textures loaded",
                    loading_state.loaded_textures
                );
            }
            return AssetLoadingStatus::Complete;
        }
    }

    AssetLoadingStatus::Loading
}
