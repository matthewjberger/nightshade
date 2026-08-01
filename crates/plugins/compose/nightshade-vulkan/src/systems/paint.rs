use crate::pre_render::Renderer;
use crate::renderer::glyph_atlas::ensure_glyph;
use crate::renderer::image_table::{ImageUpload, upload_image};
use crate::renderer::paint::{LabelRange, PAINT_PASS, PaintPass, TextDraw, create_pass, hand_over};
use crate::renderer::{VulkanRenderer, install_swapchain_pass, pass_setup, pass_with_glyphs};
use crate::resources::{CachedLabel, TextMeshCache};
use nightshade::prelude::*;
use nightshade_paint::{
    DrawText, PaintFrame, UiTextVertex, draw_order, image_instance, rect_instance, text_instance,
    text_signature,
};
use nightshade_text::contract::data::font_engine::FontEngine;
use nightshade_text::contract::data::text_data::FontKind;
use nightshade_text::contract::data::text_mesh::{
    TextMeshContext, TextMeshParams, generate_text_mesh_screen,
};
use nightshade_text::contract::resources::TextState;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_swapchain_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the paint pass was not installed: {error}"),
    }
}

pub fn gather(
    mut renderer: Renderer,
    mut frame: ResMut<PaintFrame>,
    text: Option<ResMut<TextState>>,
    mut cache: ResMut<TextMeshCache>,
) {
    let extent = renderer.swapchain.extent;
    let screen = (extent.width as f32, extent.height as f32);

    let mut rects: Vec<&nightshade_paint::DrawRect> = frame.rects.iter().collect();
    rects.sort_by(|first, second| draw_order(first, second));
    let rect_layers: Vec<u8> = rects.iter().map(|rect| rect.layer).collect();
    let instances = rects.iter().map(|rect| rect_instance(rect)).collect();

    let mut images: Vec<&nightshade_paint::DrawImage> = frame.images.iter().collect();
    images.sort_by(|first, second| match first.layer.cmp(&second.layer) {
        std::cmp::Ordering::Equal => first.z_index.cmp(&second.z_index),
        other => other,
    });
    let image_layers: Vec<u8> = images.iter().map(|image| image.layer).collect();
    let image_instances = images.iter().map(|image| image_instance(image)).collect();
    let uploads = std::mem::take(&mut frame.image_uploads);

    let mut labels = std::mem::take(&mut frame.text_meshes);
    let Some(mut text) = text else {
        frame.text_meshes = labels;
        return;
    };
    let Some((context, glyphs, pass)) = pass_with_glyphs(&mut renderer, PAINT_PASS) else {
        frame.text_meshes = labels;
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<PaintPass>() else {
        frame.text_meshes = labels;
        return;
    };

    for upload in &uploads {
        if let Err(error) = upload_image(
            &mut pass.images,
            context,
            upload.layer,
            ImageUpload {
                width: upload.width,
                height: upload.height,
                pixels: &upload.rgba_data,
                srgb: false,
                mips: false,
                format: nightshade_render::data::decoded_image::TexelFormat::Rgba8,
                levels: &[],
            },
        ) {
            tracing::error!("an interface image was not uploaded: {error}");
        }
    }

    let mut live = std::collections::HashSet::new();
    for label in &mut labels {
        let signature = text_signature(label);
        match label.entity {
            Some(entity) => {
                live.insert(entity);
                if let Some(cached) = cache.labels.get(&entity)
                    && cached.signature == signature
                {
                    if label.mesh.vertices.is_empty() {
                        label.mesh = cached.mesh.clone();
                    }
                    continue;
                }
            }
            None if !label.mesh.vertices.is_empty() => continue,
            None => {}
        }
        let params = TextMeshParams {
            font_size: label.font_size,
            line_height: 1.2,
            font_kind: match label.font_kind {
                FontKind::IconsMaterial | FontKind::IconsLucide => label.font_kind,
                _ if label.monospace => FontKind::Mono,
                kind => kind,
            },
            alignment: label.alignment,
            vertical_alignment: label.vertical_alignment,
            anchor_character: None,
        };
        label.mesh = generate_text_mesh_screen(
            &label.text,
            &mut TextMeshContext {
                font_engine: &mut text.font_engine,
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
            label.wrap_width,
        );
        if let Some(entity) = label.entity {
            cache.labels.insert(
                entity,
                CachedLabel {
                    signature,
                    mesh: label.mesh.clone(),
                },
            );
        }
    }
    cache.labels.retain(|entity, _| live.contains(entity));

    let mut ordered: Vec<&DrawText> = labels.iter().collect();
    ordered.sort_by(|first, second| match first.layer.cmp(&second.layer) {
        std::cmp::Ordering::Equal => first.z_index.cmp(&second.z_index),
        other => other,
    });

    hand_over(
        pass,
        instances,
        &rect_layers,
        image_instances,
        &image_layers,
        build_text(&ordered),
        screen,
    );
    frame.text_meshes = labels;
}

fn build_text(labels: &[&DrawText]) -> TextDraw {
    let mut draw = TextDraw::default();
    for label in labels {
        if label.mesh.vertices.is_empty() {
            continue;
        }
        let base_vertex = draw.vertices.len() as u32;
        let base_character = draw.character_colors.len() as u32;
        let index_start = draw.indices.len() as u32;
        let instance_index = draw.instances.len() as u32;

        draw.instances.push(text_instance(label));
        for vertex in &label.mesh.vertices {
            draw.vertices.push(UiTextVertex {
                position: [vertex.position.x, vertex.position.y, vertex.position.z],
                tex_coords: [vertex.tex_coords.x, vertex.tex_coords.y],
                character_index: base_character + vertex.character_index,
                text_instance_index: instance_index,
                flags: vertex.flags,
            });
        }
        for element in &label.mesh.indices {
            draw.indices.push(base_vertex + element);
        }
        draw.label_ranges.push(LabelRange {
            layer: label.layer,
            index_start,
            index_count: draw.indices.len() as u32 - index_start,
        });

        let characters = label
            .mesh
            .vertices
            .iter()
            .map(|vertex| vertex.character_index as usize + 1)
            .max()
            .unwrap_or(0)
            .max(label.text.chars().count());
        for character in 0..characters {
            draw.character_colors
                .push(character_color(&label.character_colors, character));
            draw.character_backgrounds.push(character_color(
                &label.character_background_colors,
                character,
            ));
        }
    }
    if draw.character_colors.is_empty() {
        draw.character_colors.push([0.0; 4]);
        draw.character_backgrounds.push([0.0; 4]);
    }
    draw
}

fn character_color(colors: &Option<Vec<Option<nalgebra_glm::Vec4>>>, index: usize) -> [f32; 4] {
    colors
        .as_ref()
        .and_then(|entries| entries.get(index).copied().flatten())
        .map(|color| [color.x, color.y, color.z, color.w])
        .unwrap_or([0.0; 4])
}
