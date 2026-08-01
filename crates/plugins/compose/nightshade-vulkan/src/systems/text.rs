use crate::pre_render::Renderer;
use crate::renderer::glyph_atlas::ensure_glyph;
use crate::renderer::text::{TEXT_PASS, TextFrame, TextRun, create_pass, hand_over};
use crate::renderer::{VulkanRenderer, install_scene_pass, pass_setup, pass_with_glyphs};
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_render::queries::projection::matrix_columns;
use nightshade_text::contract::RenderText;
use nightshade_text::contract::components::{Text, TextProperties};
use nightshade_text::contract::data::font_engine::FontEngine;
use nightshade_text::contract::data::text_mesh::{
    TextMeshContext, generate_text_mesh_world, text_mesh_params,
};
use nightshade_text::contract::resources::TextState;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the text pass was not installed: {error}"),
    }
}

pub fn build_text_meshes(
    mut renderer: Renderer,
    text_state: Option<ResMut<TextState>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut dirty: Vec<(Entity, usize, TextProperties)> = Vec::new();
    for (entity, text) in storage.query_ref::<&Text>().iter() {
        if text.dirty {
            dirty.push((entity, text.text_index, text.properties.clone()));
        }
    }
    if dirty.is_empty() {
        return;
    }
    let Some(mut text_state) = text_state else {
        return;
    };

    let Some((context, glyphs, _)) = pass_with_glyphs(&mut renderer, TEXT_PASS) else {
        return;
    };
    for (entity, text_index, properties) in dirty {
        let Some(content) = text_state
            .cache
            .get_text(text_index)
            .map(|content| content.to_string())
        else {
            continue;
        };
        let params = text_mesh_params(&properties);
        let mesh = generate_text_mesh_world(
            &content,
            &mut TextMeshContext {
                font_engine: &mut text_state.font_engine,
                resolve_glyph: &mut |engine: &mut FontEngine, cache_key| {
                    ensure_glyph(
                        glyphs,
                        context,
                        &mut engine.font_system,
                        &mut engine.swash_cache,
                        cache_key,
                    )
                },
            },
            &params,
        );
        if let Some(text) = storage.get_mut::<Text>(entity) {
            text.cached_mesh = Some(mesh);
            text.dirty = false;
        }
    }
}

pub fn upload_text(mut renderer: Renderer, texts: Query<(&RenderText,)>) {
    let mut data = TextFrame::default();
    texts.for_each(|_entity, (text,)| {
        if text.mesh.indices.is_empty() {
            return;
        }
        let vertex_offset = data.vertices.len() as i32;
        let index_offset = data.indices.len() as u32;
        let character_offset = data.character_colors.len() as u32;
        data.vertices.extend_from_slice(&text.mesh.vertices);
        data.indices.extend_from_slice(&text.mesh.indices);
        for index in 0..text.char_count {
            let color = text
                .character_colors
                .as_ref()
                .and_then(|colors| colors.get(index).copied().flatten())
                .map(|color| [color.x, color.y, color.z, color.w])
                .unwrap_or([0.0; 4]);
            data.character_colors.push(color);
        }
        data.runs.push(TextRun {
            model: matrix_columns(&text.transform),
            color: [text.color.x, text.color.y, text.color.z, text.color.w],
            outline_color: [
                text.outline_color.x,
                text.outline_color.y,
                text.outline_color.z,
                text.outline_color.w,
            ],
            outline_width: text.outline_width,
            smoothing: text.smoothing,
            billboard: text.billboard,
            vertex_offset,
            index_offset,
            index_count: text.mesh.indices.len() as u32,
            character_offset,
        });
    });

    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == TEXT_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(pass, data);
    }
}
