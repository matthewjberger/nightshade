use nightshade_store::prelude::*;

use crate::commands::library::start_scan;
use crate::commands::notice::report;
use crate::data::{FILE, FOLDER, PanelTab, RECENT_LIMIT, ViewerAction};
use crate::queries::library::default_folder;
use crate::queries::render::{pbr_mode, tonemap};
use crate::resources::{Settings, ViewerResources};

pub fn keep_settings(viewer: &mut ViewerResources) {
    if let Err(error) = write_settings(&viewer.settings) {
        report(
            &viewer.incoming.notice,
            format!("Your settings were not written.\n\n{error}"),
        );
    }
}

pub fn apply_settings(stage: &mut crate::resources::Stage<'_>, settings: &Settings) {
    let preview = stage.arguments.preview;
    {
        let render = &mut *stage.settings;
        render.show_sky = settings.show_sky;
        render.color_grading.exposure = settings.exposure;
        render.color_grading.tonemap_algorithm = tonemap(settings.tonemap);
    }
    {
        let debug = &mut *stage.debug_draw;
        debug.show_grid = settings.grid;
        debug.show_normals = settings.show_normals;
        debug.show_bounding_volumes = settings.show_bounds;
        debug.pbr_debug_mode = pbr_mode(settings.pbr_debug);
    }
    let viewer = &mut *stage.viewer;
    viewer.view.scene_open = settings.scene_open;
    viewer.view.inspector_open = settings.inspector_open;
    viewer.view.panel_tab = match settings.panel_tab {
        1 => PanelTab::Render,
        2 => PanelTab::Stats,
        _ => PanelTab::Scene,
    };
    viewer.view.resolution = settings.resolution;
    viewer.view.grid = settings.grid;
    viewer.view.snap = settings.snap;
    viewer.camera_input.turntable = settings.turntable;
    viewer.view.shading = settings.shading;
    viewer.view.pbr_debug = settings.pbr_debug;
    viewer.view.tonemap = settings.tonemap;
    viewer.view.exposure = settings.exposure;
    viewer.view.show_sky = settings.show_sky;
    viewer.view.show_normals = settings.show_normals;
    viewer.view.show_bounds = settings.show_bounds;
    viewer.skeleton.enabled = settings.show_skeleton;
    for path in &settings.recent {
        if !viewer.recent.contains(path) {
            viewer.recent.push(path.clone());
        }
    }
    viewer.recent.truncate(RECENT_LIMIT);
    if !preview {
        viewer.library.built_in = default_folder();
        viewer.library.folders = settings.folders.clone();
        viewer
            .library
            .folders
            .retain(|folder| Some(folder) != viewer.library.built_in.as_ref());
        viewer.library.files = settings.assets.clone();
        viewer.library.dirty = true;
        start_scan(viewer);
    }
    viewer.view.recent_dirty = true;
    viewer.view.render_panel_dirty = true;
    viewer.actions.extend([
        ViewerAction::SetShadingMode(settings.shading),
        ViewerAction::SetAtmosphere(settings.atmosphere),
        ViewerAction::SetSsao(settings.ssao),
        ViewerAction::SetBloom(settings.bloom),
        ViewerAction::SetSsr(settings.ssr),
        ViewerAction::SetSsgi(settings.ssgi),
        ViewerAction::SetAntiAliasing(settings.anti_aliasing),
        ViewerAction::SetRenderScale(settings.render_scale),
        ViewerAction::SetFogStart(settings.fog_start),
        ViewerAction::SetFogEnd(settings.fog_end),
        ViewerAction::SetFogColor(settings.fog_color),
        ViewerAction::SetFog(settings.fog),
        ViewerAction::SetFocusDistance(settings.focus_distance),
        ViewerAction::SetFocusRange(settings.focus_range),
        ViewerAction::SetDepthOfField(settings.depth_of_field),
    ]);
}

fn write_settings(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let text = toml::to_string_pretty(settings)?;
    if store_write_text(FOLDER, FILE, &text) {
        return Ok(());
    }
    Err("there is no place on this machine to keep them".into())
}
