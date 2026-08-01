use nightshade_store::prelude::*;

use crate::data::{FILE, FOLDER, PanelTab};
use crate::resources::{Settings, ViewerResources};

pub fn current_settings(viewer: &ViewerResources) -> Settings {
    Settings {
        scene_open: viewer.view.scene_open,
        inspector_open: viewer.view.inspector_open,
        panel_tab: match viewer.view.panel_tab {
            PanelTab::Scene => 0,
            PanelTab::Render => 1,
            PanelTab::Stats => 2,
        },
        resolution: viewer.view.resolution,
        grid: viewer.view.grid,
        snap: viewer.view.snap,
        turntable: viewer.camera_input.turntable,
        shading: viewer.view.shading,
        pbr_debug: viewer.view.pbr_debug,
        tonemap: viewer.view.tonemap,
        exposure: viewer.view.exposure,
        atmosphere: if viewer.view.hdr_loaded {
            viewer.settings.atmosphere
        } else {
            viewer.view.atmosphere
        },
        show_sky: viewer.view.show_sky,
        show_normals: viewer.view.show_normals,
        show_bounds: viewer.view.show_bounds,
        show_skeleton: viewer.skeleton.enabled,
        ssao: viewer.view.ssao,
        bloom: viewer.view.bloom,
        ssr: viewer.view.ssr,
        ssgi: viewer.view.ssgi,
        anti_aliasing: viewer.view.anti_aliasing,
        render_scale: viewer.view.render_scale,
        fog: viewer.view.fog,
        fog_start: viewer.view.fog_start,
        fog_end: viewer.view.fog_end,
        fog_color: viewer.view.fog_color,
        depth_of_field: viewer.view.depth_of_field,
        focus_distance: viewer.view.focus_distance,
        focus_range: viewer.view.focus_range,
        recent: viewer.recent.clone(),
        folders: viewer.library.folders.clone(),
        assets: viewer.library.files.clone(),
    }
}

pub fn read_settings() -> Option<Settings> {
    toml::from_str(&store_read_text(FOLDER, FILE)?).ok()
}
