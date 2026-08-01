use crate::pre_render::Renderer;
use crate::renderer::decals::{DECALS_PASS, DecalFrameData, GpuDecal, create_pass, hand_over};
use crate::renderer::{VulkanRenderer, install_decal_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_decals::contract::DecalFrame;
use nightshade_render::queries::projection::matrix_columns;
use nightshade_render::resources::texture_cache::TextureCache;

const NO_TEXTURE: f32 = -1.0;

pub fn install(renderer: &mut VulkanRenderer) {
    let texture_layout = renderer.textures.table.layout;
    let setup = pass_setup(renderer);
    match create_pass(&setup, texture_layout) {
        Ok(pass) => install_decal_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the decals pass was not installed: {error}"),
    }
}

pub fn upload_decals(
    mut renderer: Renderer,
    frame: Option<Res<DecalFrame>>,
    textures: Res<TextureCache>,
) {
    let Some(frame) = frame else {
        return;
    };
    let Some(view) = renderer.view.clone() else {
        return;
    };
    let view_projection = view.view_projection;

    let decals: Vec<GpuDecal> = frame
        .decals
        .iter()
        .filter(|decal| decal.visible)
        .filter_map(|decal| {
            let inverse_model = decal.transform.try_inverse()?;
            Some(GpuDecal {
                model: matrix_columns(&decal.transform),
                inverse_model: matrix_columns(&inverse_model),
                color: decal.color,
                emissive: [1.0, 1.0, 1.0, decal.emissive_strength],
                size_depth: [decal.size.x, decal.size.y, decal.depth, 0.0],
                params: [
                    slot(&textures, decal.emissive_texture.as_deref()),
                    decal.fade_start,
                    decal.fade_end,
                    slot(&textures, decal.texture.as_deref()),
                ],
            })
        })
        .collect();

    let camera_position = view.camera_position;
    let data = DecalFrameData {
        decals,
        view_projection: matrix_columns(&view_projection),
        inverse_view_projection: matrix_columns(&view.inverse_view_projection),
        camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
        screen: [
            view.screen_size.0 as f32,
            view.screen_size.1 as f32,
            0.0,
            0.0,
        ],
    };

    let table = renderer.textures.table.set;
    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == DECALS_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(pass, data, table);
    }
}

fn slot(textures: &TextureCache, name: Option<&str>) -> f32 {
    let Some(name) = name else {
        return NO_TEXTURE;
    };
    match nightshade_render::queries::texture_cache::texture_cache_texture_id(textures, name) {
        Some(id) => id.index as f32,
        None => NO_TEXTURE,
    }
}
