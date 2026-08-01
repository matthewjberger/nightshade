#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use nightshade_store::prelude::store_read;

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::commands::notice::report;
#[cfg(not(target_arch = "wasm32"))]
use crate::data::{LibraryKind, PREVIEW_FOLDER};
#[cfg(not(target_arch = "wasm32"))]
use crate::queries::library::{preview_key, preview_name};
use crate::queries::library::{roots, scan};
use crate::resources::ViewerResources;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn remember_folder(viewer: &mut ViewerResources, path: &str) {
    if viewer.library.built_in.as_deref() == Some(path)
        || viewer.library.folders.iter().any(|held| held == path)
    {
        return;
    }
    viewer.library.folders.push(path.to_string());
    viewer.library.folders.sort();
    viewer.library.dirty = true;
    start_scan(viewer);
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn remember_file(viewer: &mut ViewerResources, path: &str) {
    if viewer.library.files.iter().any(|held| held == path) {
        return;
    }
    viewer.library.files.push(path.to_string());
    viewer.library.files.sort();
    viewer.library.dirty = true;
    start_scan(viewer);
}

pub fn forget_path(viewer: &mut ViewerResources, path: &str) {
    viewer.library.folders.retain(|held| held != path);
    viewer.library.files.retain(|held| held != path);
    viewer.library.dirty = true;
    start_scan(viewer);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_scan(viewer: &mut ViewerResources) {
    if viewer.library.scanning {
        return;
    }
    viewer.library.scanning = true;
    let folders = roots(viewer);
    let files = viewer.library.files.clone();
    let scanned = Arc::clone(&viewer.library.scanned);
    let mut crew = viewer.incoming.crew.clone();
    crate::commands::crew::hire(&mut crew, move || {
        let entries = scan(&folders, &files);
        if let Ok(mut slot) = scanned.lock() {
            *slot = Some(entries);
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn start_scan(viewer: &mut ViewerResources) {
    let entries = scan(&roots(viewer), &viewer.library.files);
    if let Ok(mut slot) = viewer.library.scanned.lock() {
        *slot = Some(entries);
    }
}

pub fn take_scan(viewer: &mut ViewerResources) {
    let Some(entries) = viewer
        .library
        .scanned
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
    else {
        return;
    };
    viewer.library.scanning = false;
    if viewer.library.entries == entries {
        return;
    }
    viewer.library.entries = entries;
    viewer.library.dirty = true;
    viewer.view.browser_dirty = true;
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn add_folder(viewer: &mut ViewerResources) {
    let Some(path) = rfd::FileDialog::new()
        .set_title("Add a folder of assets to the library")
        .pick_folder()
    else {
        return;
    };
    remember_folder(viewer, &path.to_string_lossy());
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn add_files(viewer: &mut ViewerResources) {
    let Some(paths) = rfd::FileDialog::new()
        .add_filter("Models and skies", &["glb", "gltf", "hdr", "zip", "fbx"])
        .set_title("Add assets to the library")
        .pick_files()
    else {
        return;
    };
    for path in paths {
        remember_file(viewer, &path.to_string_lossy());
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn add_folder(viewer: &mut ViewerResources) {
    report(
        &viewer.incoming.notice,
        "There is no folder to read from here. Use the other library tabs.",
    );
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn add_files(viewer: &mut ViewerResources) {
    report(
        &viewer.incoming.notice,
        "There is no file system to read from here. Use the other library tabs.",
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn request_previews(viewer: &mut ViewerResources, wanted: &[String]) {
    const PREVIEWS_PER_FRAME: usize = 3;

    let mut started = 0;
    for key in wanted {
        if started >= PREVIEWS_PER_FRAME {
            break;
        }
        if viewer.browsers.slots.contains_key(key) || viewer.browsers.requested.contains(key) {
            continue;
        }
        started += 1;
        match preview_name(key) {
            Some(name) => {
                let Some(bytes) = store_read(PREVIEW_FOLDER, name) else {
                    queue_render(viewer, key);
                    continue;
                };
                viewer.browsers.requested.insert(key.clone());
                let mut crew = viewer.incoming.crew.clone();
                decode_thumbnail(&mut crew, &viewer.browsers.decoded, key.clone(), bytes);
            }
            None => {
                viewer.browsers.requested.insert(key.clone());
                let decoded = Arc::clone(&viewer.browsers.decoded);
                let path = key.clone();
                let mut crew = viewer.incoming.crew.clone();
                crate::commands::crew::hire(&mut crew, move || {
                    if let Ok(bytes) = std::fs::read(&path) {
                        decode_now(&decoded, path, bytes);
                    }
                });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn queue_render(viewer: &mut ViewerResources, key: &str) {
    let Some(path) = viewer.library.sources.get(key).cloned() else {
        return;
    };
    if viewer.library.drawn.contains(key) || viewer.library.jobs.contains(&path) {
        return;
    }
    viewer.library.jobs.push(path);
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_thumbnail(
    crew: &mut crate::resources::Crew,
    decoded: &Arc<Mutex<Vec<crate::resources::ThumbnailImage>>>,
    key: String,
    bytes: Vec<u8>,
) {
    let decoded = Arc::clone(decoded);
    crate::commands::crew::hire(crew, move || decode_now(&decoded, key, bytes));
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_now(
    decoded: &Arc<Mutex<Vec<crate::resources::ThumbnailImage>>>,
    key: String,
    bytes: Vec<u8>,
) {
    use crate::theme::THUMBNAIL_SIZE;

    let Ok(picture) = image::load_from_memory(&bytes) else {
        return;
    };
    let thumbnail = picture.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE).to_rgba8();
    let (width, height) = thumbnail.dimensions();
    if let Ok(mut queue) = decoded.lock() {
        queue.push(crate::resources::ThumbnailImage {
            key,
            pixels: thumbnail.into_raw(),
            width,
            height,
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub fn request_previews(_viewer: &mut ViewerResources, _wanted: &[String]) {}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn start_previews(viewer: &mut ViewerResources) {
    use crate::data::{PREVIEW_FRAME, PREVIEW_JOBS, PREVIEW_PIXELS};

    while viewer.library.running.len() < PREVIEW_JOBS && !viewer.library.jobs.is_empty() {
        let path = viewer.library.jobs.remove(0);
        let key = preview_key(&path);
        let Some(name) = preview_name(&key).map(|name| name.to_string()) else {
            continue;
        };
        let Ok(program) = std::env::current_exe() else {
            return;
        };
        let file = std::env::temp_dir().join(format!("nightshade-preview-{name}"));
        let kind = if matches!(LibraryKind::of(&path), Some(LibraryKind::Sky)) {
            "--sky"
        } else {
            "--model"
        };
        let mut command = std::process::Command::new(program);
        command
            .arg("--preview")
            .arg("--no-agent")
            .arg(kind)
            .arg(&path)
            .arg("--capture")
            .arg(&file)
            .arg("--capture-frame")
            .arg(PREVIEW_FRAME.to_string())
            .arg("--size")
            .arg(format!("{PREVIEW_PIXELS}x{PREVIEW_PIXELS}"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        match command.spawn() {
            Ok(child) => viewer.library.running.push(crate::resources::PreviewRun {
                child,
                key,
                file,
                started: nightshade::prelude::Instant::now(),
            }),
            Err(_) => {
                viewer.library.drawn.insert(key);
            }
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn collect_previews(viewer: &mut ViewerResources) {
    use nightshade_store::prelude::store_write;

    use crate::data::PREVIEW_GIVE_UP;

    let mut done = Vec::new();
    for (index, run) in viewer.library.running.iter_mut().enumerate() {
        if matches!(run.child.try_wait(), Ok(None)) {
            if run.started.elapsed().as_secs() < PREVIEW_GIVE_UP {
                continue;
            }
            let _ = run.child.kill();
        }
        done.push(index);
    }
    for index in done.into_iter().rev() {
        let run = viewer.library.running.remove(index);
        viewer.library.drawn.insert(run.key.clone());
        if let Some(name) = preview_name(&run.key)
            && let Ok(bytes) = std::fs::read(&run.file)
        {
            store_write(PREVIEW_FOLDER, name, &bytes);
            viewer.view.browser_dirty = true;
        }
        let _ = std::fs::remove_file(&run.file);
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn start_previews(_viewer: &mut ViewerResources) {}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn collect_previews(_viewer: &mut ViewerResources) {}
