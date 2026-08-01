use crate::commands::{load_atlas, register_item_meshes};
use crate::queries::decode_atlas;
use crate::resources::Art;
use nightshade::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::prelude::{MeshCache, MeshRenderState};

pub fn repaint_the_blocks(
    mut art: ResMut<Art>,
    mut mesh_cache: ResMut<MeshCache>,
    mut mesh_state: ResMut<MeshRenderState>,
    mut load: TextureLoad,
) {
    if art.painted == art.palette {
        return;
    }
    let palette = art.palette;
    art.painted = palette;
    let (pixels, width, height) = decode_atlas(palette);
    register_item_meshes(&mut mesh_cache, &pixels, width);
    mesh_state.full_rebuild_needed = true;
    load_atlas(&mut load, pixels, width, height);
}
