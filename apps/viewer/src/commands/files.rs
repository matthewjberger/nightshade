#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
use crate::commands::library::{remember_file, start_scan};
#[cfg(not(target_arch = "wasm32"))]
use crate::commands::load::{open_path, remember};
use crate::commands::notice::report;
use crate::data::TextureSlot;
#[cfg(not(target_os = "android"))]
use crate::queries::files::is_picture;
#[cfg(not(target_arch = "wasm32"))]
use crate::queries::files::read_path;
use crate::queries::files::{is_fbx, unzip};
use crate::resources::{Incoming, PendingAsset, TextureDrop, ViewerResources};

#[cfg(not(target_os = "android"))]
const PICTURES: [&str; 9] = [
    "png", "jpg", "jpeg", "webp", "tga", "bmp", "tif", "tiff", "ktx2",
];

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn pick_texture(viewer: &mut ViewerResources, slot: TextureSlot) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Pictures", &PICTURES)
        .set_title("Open a picture for this slot")
        .pick_file()
    else {
        return;
    };
    let Some((name, bytes)) = read_path(&path) else {
        report(
            &viewer.incoming.notice,
            "That picture could not be read from the disk.",
        );
        return;
    };
    if !is_picture(&name) {
        report(
            &viewer.incoming.notice,
            format!("{name} is not a picture the viewer can read."),
        );
        return;
    }
    offer_texture(&viewer.incoming, TextureDrop { slot, name, bytes });
}

#[cfg(target_arch = "wasm32")]
pub fn pick_texture(viewer: &mut ViewerResources, slot: TextureSlot) {
    let inbox = viewer.incoming.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Pictures", &PICTURES)
            .set_title("Open a picture for this slot")
            .pick_file()
            .await
        else {
            return;
        };
        let name = handle.file_name();
        if !is_picture(&name) {
            report(
                &inbox.notice,
                format!("{name} is not a picture the viewer can read."),
            );
            return;
        }
        let bytes = handle.read().await;
        offer_texture(&inbox, TextureDrop { slot, name, bytes });
    });
}

#[cfg(target_os = "android")]
pub fn pick_texture(viewer: &mut ViewerResources, _slot: TextureSlot) {
    report(
        &viewer.incoming.notice,
        "There is no file dialog here. Drop a picture onto the slot instead.",
    );
}

pub fn offer_texture(inbox: &Incoming, drop: TextureDrop) {
    if let Ok(mut slot) = inbox.texture.lock() {
        *slot = Some(drop);
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn screenshot(stage: &mut crate::resources::Stage<'_>) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Picture", &["png"])
        .set_file_name("viewer.png")
        .set_title("Write the picture to")
        .save_file()
    else {
        return;
    };
    let path = if path.extension().is_some() {
        path
    } else {
        path.with_extension("png")
    };
    nightshade_graphics::commands::queues::capture_screenshot_to_path(stage.queues, path);
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn screenshot(stage: &mut crate::resources::Stage<'_>) {
    report(
        &stage.viewer.incoming.notice,
        "There is no file to write a picture to here.",
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_kept_path(stage: &mut crate::resources::Stage<'_>, path: &str) {
    if !std::path::Path::new(path).exists() {
        let viewer = &mut *stage.viewer;
        viewer.recent.retain(|held| held != path);
        viewer.view.recent_dirty = true;
        report(
            &viewer.incoming.notice,
            format!("{path} is not there any more."),
        );
        return;
    }
    open_path(stage, path);
}

#[cfg(target_arch = "wasm32")]
pub fn open_kept_path(stage: &mut crate::resources::Stage<'_>, path: &str) {
    report(
        &stage.viewer.incoming.notice,
        format!("There is no file named {path} here."),
    );
}

#[cfg(not(target_os = "android"))]
const EXTENSIONS: [&str; 5] = ["glb", "gltf", "hdr", "zip", "fbx"];

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn open_dialog(viewer: &mut ViewerResources) {
    let inbox = viewer.incoming.clone();
    let Some(paths) = rfd::FileDialog::new()
        .add_filter("Models and skies", &EXTENSIONS)
        .set_title("Open a model or a sky")
        .pick_files()
    else {
        return;
    };
    for path in &paths {
        remember(viewer, &path.to_string_lossy());
    }
    let files: Vec<(String, Vec<u8>)> = paths.iter().filter_map(|path| read_path(path)).collect();
    if files.is_empty() {
        report(
            &inbox.notice,
            "Those files could not be read from the disk.",
        );
        return;
    }
    route_drop(&inbox, files);
}

#[cfg(target_arch = "wasm32")]
pub fn open_dialog(viewer: &mut ViewerResources) {
    let inbox = viewer.incoming.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let Some(handles) = rfd::AsyncFileDialog::new()
            .add_filter("Models and skies", &EXTENSIONS)
            .set_title("Open a model or a sky")
            .pick_files()
            .await
        else {
            return;
        };
        let mut files = Vec::with_capacity(handles.len());
        for handle in handles {
            files.push((handle.file_name(), handle.read().await));
        }
        if files.is_empty() {
            report(&inbox.notice, "Those files could not be read.");
            return;
        }
        route_drop(&inbox, files);
    });
}

#[cfg(target_os = "android")]
pub fn open_dialog(viewer: &mut ViewerResources) {
    report(
        &viewer.incoming.notice,
        "There is no file dialog here. Open a model from the library.",
    );
}

pub fn route_drop(inbox: &Incoming, mut files: Vec<(String, Vec<u8>)>) {
    let mut pack_name = files
        .first()
        .map(|(name, _)| name.clone())
        .unwrap_or_default();

    if files.len() == 1
        && files[0].0.to_lowercase().ends_with(".zip")
        && let Some(contents) = unzip(&files[0].1)
    {
        files = contents;
    }

    let model = files.iter().position(|(name, _)| {
        let lowered = name.to_lowercase();
        lowered.ends_with(".glb") || lowered.ends_with(".gltf")
    });

    if let Some(index) = model {
        let (name, gltf) = files.remove(index);
        let asset = if name.to_lowercase().ends_with(".glb") || files.is_empty() {
            PendingAsset::Model(gltf)
        } else {
            let directory = name
                .rsplit_once('/')
                .map(|(above, _)| format!("{above}/"))
                .unwrap_or_default();
            let resources = files
                .into_iter()
                .map(|(path, bytes)| {
                    let key = path.strip_prefix(&directory).unwrap_or(&path).to_string();
                    (key, bytes)
                })
                .collect();
            PendingAsset::ModelWithResources { gltf, resources }
        };
        accept(inbox, &name, asset);
        return;
    }

    if files.iter().any(|(name, _)| is_fbx(name)) {
        if files.len() == 1 {
            pack_name = files[0].0.clone();
        }
        crate::commands::blender::convert_pack(inbox, &pack_name, files);
        return;
    }

    if let Some(index) = files
        .iter()
        .position(|(name, _)| name.to_lowercase().ends_with(".hdr"))
    {
        let (name, bytes) = files.remove(index);
        accept(inbox, &name, PendingAsset::Sky(bytes));
    }
}

fn accept(inbox: &Incoming, name: &str, asset: PendingAsset) {
    if let Ok(mut slot) = inbox.asset.lock() {
        *slot = Some(asset);
    }
    if let Ok(mut guard) = inbox.loading.lock() {
        *guard = Some(name.to_string());
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn export_dialog(stage: &mut crate::resources::Stage<'_>, storage: &nightshade::ecs::Storage) {
    let notice = stage.viewer.incoming.notice.clone();
    if stage.viewer.model.roots.is_empty() {
        report(&notice, "There is no model open to write.");
        return;
    }

    let mut dialog = rfd::FileDialog::new()
        .add_filter("Model", &["glb"])
        .set_file_name("scene.glb")
        .set_title("Write the model to");
    if let Some(folder) = stage.viewer.library.folders.first().cloned() {
        dialog = dialog.set_directory(folder);
    }
    let Some(path) = dialog.save_file() else {
        return;
    };
    let path = if path.extension().is_some() {
        path
    } else {
        path.with_extension("glb")
    };

    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "scene".to_string());

    let scene = match gather_scene(stage, storage, &name) {
        Ok(scene) => scene,
        Err(message) => {
            report(&notice, message);
            return;
        }
    };

    let clips = scene.animations.len();
    let nodes = scene.nodes.len();
    match nightshade_gltf::export_glb(&scene) {
        Ok(bytes) => match std::fs::write(&path, &bytes) {
            Ok(()) => {
                let written = path.to_string_lossy().to_string();
                let viewer = &mut *stage.viewer;
                if viewer
                    .library
                    .folders
                    .iter()
                    .any(|folder| written.starts_with(folder.as_str()))
                {
                    start_scan(viewer);
                } else {
                    remember_file(viewer, &written);
                }
                report(
                    &notice,
                    format!("Wrote {nodes} nodes and {clips} clips to {written}."),
                )
            }
            Err(error) => report(&notice, format!("The model did not write.\n\n{error}")),
        },
        Err(message) => report(&notice, format!("The model did not write.\n\n{message}")),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn export_dialog(stage: &mut crate::resources::Stage<'_>, storage: &nightshade::ecs::Storage) {
    let notice = stage.viewer.incoming.notice.clone();
    let scene = match gather_scene(stage, storage, "scene") {
        Ok(scene) => scene,
        Err(message) => {
            report(&notice, message);
            return;
        }
    };
    let nodes = scene.nodes.len();
    let clips = scene.animations.len();
    let bytes = match nightshade_gltf::export_glb(&scene) {
        Ok(bytes) => bytes,
        Err(message) => {
            report(
                &notice,
                format!(
                    "The model did not write.

{message}"
                ),
            );
            return;
        }
    };
    wasm_bindgen_futures::spawn_local(async move {
        let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Model", &["glb"])
            .set_file_name("scene.glb")
            .set_title("Write the model to")
            .save_file()
            .await
        else {
            return;
        };
        match handle.write(&bytes).await {
            Ok(()) => report(&notice, format!("Wrote {nodes} nodes and {clips} clips.")),
            Err(error) => report(
                &notice,
                format!(
                    "The model did not write.

{error}"
                ),
            ),
        }
    });
}

#[cfg(target_os = "android")]
pub fn export_dialog(stage: &mut crate::resources::Stage<'_>, _storage: &nightshade::ecs::Storage) {
    report(
        &stage.viewer.incoming.notice,
        "There is no file to write a model to here.",
    );
}

#[cfg(not(target_os = "android"))]
pub fn gather_scene(
    stage: &crate::resources::Stage<'_>,
    storage: &nightshade::ecs::Storage,
    name: &str,
) -> Result<nightshade_gltf::data::SceneExport, String> {
    use crate::queries::export::gather;

    gather(
        stage.viewer,
        storage,
        stage.hierarchy,
        stage.meshes,
        stage.materials,
        stage.sources,
        name,
    )
}
