use std::collections::{HashMap, HashSet};

use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_prefab::{PrefabSpawn, data::ModelImport};
use nightshade_render::resources::texture_cache::TextureOwner;
use nightshade_scene::contract::LocalTransform;

use crate::commands::animation::hold_clips;
use crate::commands::notice::report;
use crate::commands::scene::build_rows;
use crate::commands::selection::select;
use crate::data::ATMOSPHERE_LABELS;
#[cfg(not(target_arch = "wasm32"))]
use crate::data::RECENT_LIMIT;
use crate::queries::load::{decode_preview, morph_meshes, ordered_entities, preview_sources};
#[cfg(target_arch = "wasm32")]
use crate::resources::PendingAsset;
#[cfg(not(target_arch = "wasm32"))]
use crate::resources::ViewerResources;
use crate::resources::{AnimationView, ClipInfo, History, LoadReport, ModelStats, ThumbnailImage};

fn release_dropped_textures(stage: &mut crate::resources::Stage<'_>) {
    let names = std::mem::take(&mut stage.viewer.model.dropped_textures);
    let cache = &mut *stage.textures;
    for name in names {
        texture_cache_release(cache, &TextureOwner::Named(name));
    }
}

const DEFAULT_MODEL: &[u8] = include_bytes!("../../../../assets/gltf/DamagedHelmet.glb");

pub fn load_default(stage: &mut crate::resources::Stage<'_>) {
    let (model, sky, bare) = {
        let arguments = stage.arguments;
        (
            arguments.model.clone(),
            arguments.sky.clone(),
            arguments.preview,
        )
    };
    match model {
        #[cfg(not(target_arch = "wasm32"))]
        Some(path) => open_path(stage, &path),
        #[cfg(target_arch = "wasm32")]
        Some(path) => load_named(stage, &path),
        None if bare => {}
        None => load_model(stage, DEFAULT_MODEL.to_vec()),
    }
    if let Some(path) = sky {
        load_named_sky(stage, &path);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_path(stage: &mut crate::resources::Stage<'_>, path: &str) {
    let lowered = path.to_lowercase();
    if lowered.ends_with(".hdr") {
        load_named_sky(stage, path);
    } else if lowered.ends_with(".zip") || lowered.ends_with(".fbx") {
        let Some(file) = crate::queries::files::read_path(std::path::Path::new(path)) else {
            let notice = stage.viewer.incoming.notice.clone();
            report(&notice, format!("{path} could not be read from the disk."));
            return;
        };
        let viewer = &mut *stage.viewer;
        remember(viewer, path);
        let inbox = viewer.incoming.clone();
        crate::commands::files::route_drop(&inbox, vec![file]);
    } else {
        load_named(stage, path);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn remember(viewer: &mut ViewerResources, path: &str) {
    viewer.recent.retain(|held| held != path);
    viewer.recent.insert(0, path.to_string());
    viewer.recent.truncate(RECENT_LIMIT);
    viewer.view.recent_dirty = true;
}

#[cfg(not(target_arch = "wasm32"))]
fn load_named(stage: &mut crate::resources::Stage<'_>, path: &str) {
    remember(stage.viewer, path);
    begin(stage, path);
    let handle = nightshade_load::load(
        stage.loading,
        nightshade_load::Source::Path(std::path::PathBuf::from(path)),
    );
    stage.viewer.model.pending = Some(handle);
}

#[cfg(target_arch = "wasm32")]
fn load_named(stage: &mut crate::resources::Stage<'_>, url: &str) {
    begin(stage, url);
    let handle =
        nightshade_load::load(stage.loading, nightshade_load::Source::Url(url.to_string()));
    stage.viewer.model.pending = Some(handle);
}

#[cfg(not(target_arch = "wasm32"))]
fn load_named_sky(stage: &mut crate::resources::Stage<'_>, path: &str) {
    remember(stage.viewer, path);
    match std::fs::read(path) {
        Ok(bytes) => load_sky(stage, bytes),
        Err(error) => {
            let notice = stage.viewer.incoming.notice.clone();
            report(&notice, format!("The sky did not open.\n\n{error}"));
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_named_sky(stage: &mut crate::resources::Stage<'_>, url: &str) {
    let inbox = stage.viewer.incoming.clone();
    let url = url.to_string();
    ehttp::fetch(ehttp::Request::get(&url), move |result| match result {
        Ok(response) if response.ok => {
            if let Ok(mut slot) = inbox.asset.lock() {
                *slot = Some(PendingAsset::Sky(response.bytes));
            }
        }
        _ => report(&inbox.notice, format!("The sky at {url} did not download.")),
    });
}

fn begin(stage: &mut crate::resources::Stage<'_>, label: &str) {
    if let Ok(mut guard) = stage.viewer.incoming.loading.lock() {
        *guard = Some(label.to_string());
    }
}

pub fn finish(stage: &mut crate::resources::Stage<'_>) {
    if let Ok(mut guard) = stage.viewer.incoming.loading.lock() {
        *guard = None;
    }
}

pub fn load_model(stage: &mut crate::resources::Stage<'_>, bytes: Vec<u8>) {
    let handle = nightshade_load::load(stage.loading, nightshade_load::Source::Bytes(bytes));
    stage.viewer.model.pending = Some(handle);
}

pub fn load_model_with_resources(
    stage: &mut crate::resources::Stage<'_>,
    gltf: Vec<u8>,
    files: HashMap<String, Vec<u8>>,
) {
    let handle = nightshade_load::load(
        stage.loading,
        nightshade_load::Source::Parts { head: gltf, files },
    );
    stage.viewer.model.pending = Some(handle);
}

pub fn load_sky(stage: &mut crate::resources::Stage<'_>, bytes: Vec<u8>) {
    stage.settings.atmosphere = Atmosphere::Hdr;
    load_hdr_skybox(stage.queues, bytes);
    let viewer = &mut *stage.viewer;
    viewer.view.hdr_loaded = true;
    viewer.view.atmosphere = ATMOSPHERE_LABELS.len() - 1;
    viewer.view.render_panel_dirty = true;
}

pub fn spawn_result(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    mut import: ModelImport,
) {
    hold_clips(stage, storage);
    despawn_current(stage, storage);

    let stats = ModelStats {
        meshes: import.meshes.len() as u32,
        vertices: import
            .meshes
            .values()
            .map(|mesh| mesh.vertices.len() as u32)
            .sum(),
        triangles: import
            .meshes
            .values()
            .map(|mesh| (mesh.indices.len() / 3) as u32)
            .sum(),
        materials: import.materials.len() as u32,
        textures: import.texture_plan.len() as u32,
        dimensions: [0.0, 0.0, 0.0],
    };
    let clips: Vec<ClipInfo> = import
        .animations
        .iter()
        .map(|clip| ClipInfo {
            name: clip.name.clone(),
            duration: clip.duration,
        })
        .collect();
    let variants = import
        .prefabs
        .first()
        .map(|prefab| prefab.material_variants.clone())
        .unwrap_or_default();
    let exposure = import.suggested_exposure;
    let previews = preview_sources(&import);

    let before: HashSet<Entity> = storage.query_entities::<(LocalTransform,)>().collect();

    let animations = import.animations.clone();
    let skins = import.skins.clone();
    let prefabs = import.prefabs.clone();
    let mut roots = Vec::new();
    {
        let mut spawn = PrefabSpawn {
            storage,
            parts: stage.prefab_parts(),
        };
        nightshade_prefab::queue_model_load(&mut spawn, &mut import);
        for prefab in &prefabs {
            roots.push(nightshade_prefab::spawn_prefab_with_skins(
                &mut spawn,
                prefab,
                &animations,
                &skins,
                Vec3::zeros(),
            ));
        }
    }
    if roots.is_empty() {
        let notice = stage.viewer.incoming.notice.clone();
        report(
            &notice,
            "The file opened but it holds no scene to show.".to_string(),
        );
        return;
    }

    let spawned: HashSet<Entity> = storage
        .query_entities::<(LocalTransform,)>()
        .filter(|entity| !before.contains(entity))
        .collect();
    let entities = ordered_entities(storage, &roots, &spawned);
    let morphs = morph_meshes(storage, &entities);

    stage.settings.color_grading.exposure = exposure;

    let viewer = &mut *stage.viewer;
    viewer.model.roots = roots;
    viewer.model.entities = entities;
    viewer.model.clips = clips;
    viewer.model.variants = variants;
    viewer.model.morphs = morphs;
    viewer.model.report = Some(LoadReport { stats, delay: 1 });
    viewer.view.active_variant = None;
    viewer.view.exposure = exposure;
    viewer.view.render_panel_dirty = true;
    let blend_time = viewer.animation.blend_time;
    viewer.animation = AnimationView {
        duration: viewer
            .model
            .clips
            .first()
            .map(|clip| clip.duration)
            .unwrap_or(0.0),
        blend_time,
        ..Default::default()
    };
    viewer.view.layers_dirty = true;
    viewer.camera_input.frame_requested = true;
    viewer.history = History::default();
    viewer.model.previews_stale = true;
    let mut crew = viewer.incoming.crew.clone();
    decode_previews(&mut crew, &viewer.model.previews_decoded, previews);
    select(viewer, storage, None);
    build_rows(viewer, storage);
}

fn decode_previews(
    crew: &mut crate::resources::Crew,
    decoded: &std::sync::Arc<std::sync::Mutex<Vec<ThumbnailImage>>>,
    sources: Vec<(String, Vec<u8>)>,
) {
    let decoded = std::sync::Arc::clone(decoded);
    crate::commands::crew::hire(crew, move || {
        for (name, bytes) in sources {
            if let Some(image) = decode_preview(name, &bytes)
                && let Ok(mut queue) = decoded.lock()
            {
                queue.push(image);
            }
        }
    });
}

fn despawn_current(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let entities = std::mem::take(&mut stage.viewer.model.entities);
    release_camera(stage, &entities);
    release_dropped_textures(stage);
    let alive: Vec<Entity> = entities
        .into_iter()
        .filter(|entity| storage.is_alive(*entity))
        .collect();
    if !alive.is_empty() {
        storage.despawn_entities(&alive);
    }
    let viewer = &mut *stage.viewer;
    viewer.model.roots.clear();
    viewer.model.rows.clear();
    viewer.model.clips.clear();
    viewer.model.variants.clear();
    viewer.model.morphs.clear();
    viewer.model.cameras.clear();
    viewer.model.stats = None;
    viewer.model.isolated = None;
    viewer.selection.others.clear();
    viewer.picking.hovered = None;
    stage.mesh_state.full_rebuild_needed = true;
}

pub fn release_camera(stage: &mut crate::resources::Stage<'_>, going: &[Entity]) {
    let fallback = stage.viewer.camera_input.viewer_camera;
    let active = &mut *stage.active_camera;
    if active.0.is_some_and(|camera| going.contains(&camera)) {
        active.0 = fallback;
    }
}
