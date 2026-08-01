use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiBuild;

use crate::commands::lists::{
    rebuild_browser, rebuild_cameras, rebuild_clips, rebuild_layers, rebuild_library_strip,
    rebuild_morphs, rebuild_recent, rebuild_scene_tree, rebuild_variants, write_render_panel,
};
use crate::commands::material_grid::rebuild_material;
use crate::commands::sections::{rebuild_camera, rebuild_light};
use crate::resources::ViewerResources;

pub fn rebuild_lists(
    mut viewer: ResMut<ViewerResources>,
    mut build: UiBuild<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;

    if std::mem::take(&mut viewer.view.tree_dirty) {
        rebuild_scene_tree(viewer, &mut build);
        rebuild_cameras(viewer, &mut build);
        rebuild_variants(viewer, &mut build);
        rebuild_morphs(viewer, &mut build, storage);
        rebuild_clips(viewer, &mut build);
    }

    if std::mem::take(&mut viewer.view.layers_dirty) {
        rebuild_layers(viewer, &mut build);
    }

    if std::mem::take(&mut viewer.view.detail_dirty) {
        rebuild_light(viewer, &mut build);
        rebuild_camera(viewer, &mut build);
        rebuild_material(viewer, &mut build);
    }

    if std::mem::take(&mut viewer.view.browser_dirty) {
        rebuild_browser(viewer, &mut build);
    }

    if std::mem::take(&mut viewer.library.dirty) {
        rebuild_library_strip(viewer, &mut build);
    }

    if std::mem::take(&mut viewer.view.recent_dirty) {
        rebuild_recent(viewer, &mut build);
    }

    if std::mem::take(&mut viewer.view.render_panel_dirty) {
        write_render_panel(viewer, &mut build);
    }
}
