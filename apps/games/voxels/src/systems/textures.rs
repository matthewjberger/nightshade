use crate::commands::decode_atlas;
use crate::commands::{load_atlas, register_item_meshes};
use crate::resources::{Art, Settings, UiHandles};
use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::ui_upload_image;

pub fn repaint_the_blocks(
    settings: Res<Settings>,
    mut art: ResMut<Art>,
    mut handles: ResMut<UiHandles>,
    mut mesh_cache: ResMut<MeshCache>,
    mut mesh_state: ResMut<MeshRenderState>,
    mut load: TextureLoad,
    mut build: UiBuild,
) {
    if art.painted == settings.palette {
        return;
    }
    art.painted = settings.palette;
    let (pixels, width, height) = decode_atlas(settings.palette);
    register_item_meshes(&mut mesh_cache, &pixels, width);
    mesh_state.full_rebuild_needed = true;

    let slot = settings.palette.code() as usize;
    if art.layers[slot].is_none() {
        let mut tree = UiTreeBuilder::new(build.context());
        let (images, runtime) = tree.images_and_runtime();
        art.layers[slot] =
            ui_upload_image(images, runtime, &pixels, width, height).map(|upload| upload.layer);
    }
    if let Some(layer) = art.layers[slot] {
        handles.atlas_layer = Some(layer);
        for index in 0..theme::HOTBAR_SLOTS {
            handles.shown[index] = None;
        }
    }
    load_atlas(&mut load, pixels, width, height);
}
