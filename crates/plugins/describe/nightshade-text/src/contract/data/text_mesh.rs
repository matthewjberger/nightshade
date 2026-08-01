use crate::contract::data::cosmic_text::{Buffer, CacheKey, LayoutGlyph};
use crate::contract::data::font_engine::FontEngine;
use crate::contract::data::glyph_packer::GlyphAlloc;
use crate::contract::data::text_data::{
    FontKind, GLYPH_COLORED, TextAlignment, TextMesh, TextVertex, VerticalAlignment,
};
use nalgebra_glm::{vec2, vec3};

const PIXELS_TO_WORLD: f32 = 0.01;

pub struct TextMeshParams {
    pub font_size: f32,
    pub line_height: f32,
    pub font_kind: FontKind,
    pub alignment: TextAlignment,
    pub vertical_alignment: VerticalAlignment,
    pub anchor_character: Option<usize>,
}

pub fn text_mesh_params(
    properties: &crate::contract::components::TextProperties,
) -> TextMeshParams {
    let font_kind = match properties.font_kind {
        FontKind::IconsMaterial | FontKind::IconsLucide => properties.font_kind,
        _ if properties.monospace_width.is_some() => FontKind::Mono,
        kind => kind,
    };
    TextMeshParams {
        font_size: properties.font_size,
        line_height: properties.line_height,
        font_kind,
        alignment: properties.alignment,
        vertical_alignment: properties.vertical_alignment,
        anchor_character: properties.anchor_character,
    }
}

pub struct TextMeshContext<'a> {
    pub font_engine: &'a mut FontEngine,
    pub resolve_glyph: &'a mut dyn FnMut(&mut FontEngine, CacheKey) -> Option<GlyphAlloc>,
}

pub fn generate_text_mesh_world(
    text: &str,
    ctx: &mut TextMeshContext<'_>,
    params: &TextMeshParams,
) -> TextMesh {
    let font_size = params.font_size;
    let line_height = font_size * params.line_height;

    let buffer = ctx
        .font_engine
        .shape_buffer(text, font_size, line_height, None, params.font_kind);

    let (max_line_width, total_height) = measure_buffer(&buffer);

    let y_origin = match params.vertical_alignment {
        VerticalAlignment::Top | VerticalAlignment::Baseline => -font_size * 0.8,
        VerticalAlignment::Middle => -total_height * 0.5,
        VerticalAlignment::Bottom => -total_height,
    };

    let use_anchor = params.anchor_character.is_some();
    let anchor_offset = if let Some(anchor_index) = params.anchor_character {
        anchor_x_offset(&buffer, anchor_index)
    } else {
        0.0
    };

    let mut mesh = TextMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        bounds: vec2(max_line_width, total_height) * PIXELS_TO_WORLD,
    };

    let runs: Vec<(f32, f32, Vec<LayoutGlyph>)> = buffer
        .layout_runs()
        .map(|run| (run.line_y, run.line_w, run.glyphs.to_vec()))
        .collect();
    drop(buffer);

    let mut character_index_counter: u32 = 0;
    for (line_y, line_w, glyphs) in runs {
        let line_x_offset = if use_anchor {
            -anchor_offset
        } else {
            match params.alignment {
                TextAlignment::Left => 0.0,
                TextAlignment::Center => -line_w * 0.5,
                TextAlignment::Right => -line_w,
            }
        };

        for glyph in glyphs {
            emit_glyph_world(
                &glyph,
                ctx,
                line_x_offset,
                line_y,
                y_origin,
                &mut mesh,
                &mut character_index_counter,
            );
        }
    }

    mesh
}

pub fn generate_text_mesh_screen(
    text: &str,
    ctx: &mut TextMeshContext<'_>,
    params: &TextMeshParams,
    wrap_width: Option<f32>,
) -> TextMesh {
    let font_size = params.font_size;
    let line_height = font_size * params.line_height;

    let buffer =
        ctx.font_engine
            .shape_buffer(text, font_size, line_height, wrap_width, params.font_kind);
    let (max_line_width, total_height) = measure_buffer(&buffer);

    let y_origin = match params.vertical_alignment {
        VerticalAlignment::Top => 0.0,
        VerticalAlignment::Middle | VerticalAlignment::Baseline => -total_height * 0.5,
        VerticalAlignment::Bottom => -total_height,
    };

    let mut mesh = TextMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        bounds: vec2(max_line_width, total_height),
    };

    let runs: Vec<(f32, f32, Vec<LayoutGlyph>)> = buffer
        .layout_runs()
        .map(|run| (run.line_y, run.line_w, run.glyphs.to_vec()))
        .collect();
    drop(buffer);

    let mut character_index_counter: u32 = 0;
    for (line_y, line_w, glyphs) in runs {
        let line_x_offset = match params.alignment {
            TextAlignment::Left => 0.0,
            TextAlignment::Center => -line_w * 0.5,
            TextAlignment::Right => -line_w,
        };

        for glyph in glyphs {
            emit_glyph_screen(
                &glyph,
                ctx,
                line_x_offset,
                line_y + y_origin,
                &mut mesh,
                &mut character_index_counter,
            );
        }
    }

    mesh
}

fn measure_buffer(buffer: &Buffer) -> (f32, f32) {
    let mut max_width = 0.0f32;
    let mut total_height = 0.0f32;
    for run in buffer.layout_runs() {
        if run.line_w > max_width {
            max_width = run.line_w;
        }
        total_height = total_height.max(run.line_top + run.line_height);
    }
    (max_width, total_height)
}

fn anchor_x_offset(buffer: &Buffer, anchor_index: usize) -> f32 {
    let mut current = 0usize;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            if current == anchor_index {
                return glyph.x + glyph.w * 0.5;
            }
            current += 1;
        }
    }
    0.0
}

fn emit_glyph_world(
    glyph: &LayoutGlyph,
    ctx: &mut TextMeshContext<'_>,
    line_x_offset: f32,
    line_y: f32,
    y_origin: f32,
    mesh: &mut TextMesh,
    character_index_counter: &mut u32,
) {
    let physical = glyph.physical((line_x_offset, line_y), 1.0);
    let cache_key = physical.cache_key;
    let alloc = (ctx.resolve_glyph)(ctx.font_engine, cache_key);

    let character_index = *character_index_counter;
    *character_index_counter += 1;

    let alloc = match alloc {
        Some(alloc) => alloc,
        None => return,
    };

    let glyph_x = physical.x as f32 + alloc.placement_left;
    let glyph_y_top = y_origin - physical.y as f32 - alloc.placement_top;

    let x = glyph_x * PIXELS_TO_WORLD;
    let y_top = -glyph_y_top * PIXELS_TO_WORLD;
    let width = alloc.size.x * PIXELS_TO_WORLD;
    let height = alloc.size.y * PIXELS_TO_WORLD;
    let y_bottom = y_top - height;

    push_quad(mesh, x, y_top, width, y_bottom, &alloc, character_index);
}

fn emit_glyph_screen(
    glyph: &LayoutGlyph,
    ctx: &mut TextMeshContext<'_>,
    line_x_offset: f32,
    line_y: f32,
    mesh: &mut TextMesh,
    character_index_counter: &mut u32,
) {
    let physical = glyph.physical((line_x_offset, line_y), 1.0);
    let cache_key = physical.cache_key;
    let alloc = (ctx.resolve_glyph)(ctx.font_engine, cache_key);

    let character_index = *character_index_counter;
    *character_index_counter += 1;

    let alloc = match alloc {
        Some(alloc) => alloc,
        None => return,
    };

    let glyph_x = physical.x as f32 + alloc.placement_left;
    let glyph_y_top = physical.y as f32 - alloc.placement_top;

    let x = glyph_x;
    let y_top = glyph_y_top;
    let width = alloc.size.x;
    let height = alloc.size.y;
    let y_bottom = y_top + height;

    push_quad(mesh, x, y_top, width, y_bottom, &alloc, character_index);
}

fn push_quad(
    mesh: &mut TextMesh,
    x: f32,
    y_top: f32,
    width: f32,
    y_bottom: f32,
    alloc: &GlyphAlloc,
    character_index: u32,
) {
    let vertex_offset = mesh.vertices.len() as u32;
    let flags = if alloc.is_color { GLYPH_COLORED } else { 0 };

    mesh.vertices.push(TextVertex {
        position: vec3(x, y_top, 0.0),
        tex_coords: vec2(alloc.pixel_min.x, alloc.pixel_min.y),
        character_index,
        flags,
    });
    mesh.vertices.push(TextVertex {
        position: vec3(x + width, y_top, 0.0),
        tex_coords: vec2(alloc.pixel_max.x, alloc.pixel_min.y),
        character_index,
        flags,
    });
    mesh.vertices.push(TextVertex {
        position: vec3(x + width, y_bottom, 0.0),
        tex_coords: vec2(alloc.pixel_max.x, alloc.pixel_max.y),
        character_index,
        flags,
    });
    mesh.vertices.push(TextVertex {
        position: vec3(x, y_bottom, 0.0),
        tex_coords: vec2(alloc.pixel_min.x, alloc.pixel_max.y),
        character_index,
        flags,
    });

    mesh.indices.extend_from_slice(&[
        vertex_offset,
        vertex_offset + 1,
        vertex_offset + 2,
        vertex_offset,
        vertex_offset + 2,
        vertex_offset + 3,
    ]);
}
