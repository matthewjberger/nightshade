use crate::data::{DrawRect, DrawText};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PaintRectInstance {
    pub position_size: [f32; 4],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub clip_rect: [f32; 4],
    pub params: [f32; 4],
    pub shadow_color: [f32; 4],
    pub shadow_params: [f32; 4],
    pub effect_params: [f32; 4],
    pub quad_corner_01: [f32; 4],
    pub quad_corner_23: [f32; 4],
    pub effect_kind: u32,
    pub is_quad: u32,
    pub padding: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextInstanceData {
    pub color: [f32; 4],
    pub outline_color: [f32; 4],
    pub clip_rect: [f32; 4],
    pub params: [f32; 4],
    pub position: [f32; 4],
}

pub fn layer_depth(layer: u8, z_index: i32) -> f32 {
    let combined = (layer as i32) * 100000 + z_index;
    0.1 + (combined as f32 / 10000000.0) * 0.8
}

pub fn draw_order(first: &DrawRect, second: &DrawRect) -> std::cmp::Ordering {
    match first.layer.cmp(&second.layer) {
        std::cmp::Ordering::Equal => first.z_index.cmp(&second.z_index),
        other => other,
    }
}

pub fn rect_instance(rect: &DrawRect) -> PaintRectInstance {
    let clip = rect
        .clip_rect
        .map(|area| [area.min.x, area.min.y, area.max.x, area.max.y])
        .unwrap_or([0.0; 4]);
    let (shadow_color, shadow_params) = match rect.shadow {
        Some(shadow) => (
            [
                shadow.color.x,
                shadow.color.y,
                shadow.color.z,
                shadow.color.w,
            ],
            [shadow.offset.x, shadow.offset.y, shadow.blur, shadow.spread],
        ),
        None => ([0.0; 4], [0.0; 4]),
    };
    let (quad_corner_01, quad_corner_23, is_quad) = match rect.quad_corners {
        Some(corners) => (
            [corners[0].x, corners[0].y, corners[1].x, corners[1].y],
            [corners[2].x, corners[2].y, corners[3].x, corners[3].y],
            1,
        ),
        None => ([0.0; 4], [0.0; 4], 0),
    };
    PaintRectInstance {
        position_size: [rect.position.x, rect.position.y, rect.size.x, rect.size.y],
        color: [rect.color.x, rect.color.y, rect.color.z, rect.color.w],
        border_color: [
            rect.border_color.x,
            rect.border_color.y,
            rect.border_color.z,
            rect.border_color.w,
        ],
        clip_rect: clip,
        params: [
            rect.corner_radius,
            rect.border_width,
            layer_depth(rect.layer, rect.z_index),
            rect.rotation,
        ],
        shadow_color,
        shadow_params,
        effect_params: rect.effect_params,
        quad_corner_01,
        quad_corner_23,
        effect_kind: rect.effect_kind,
        is_quad,
        padding: [0; 2],
    }
}

pub fn text_instance(text: &DrawText) -> TextInstanceData {
    let clip = text
        .clip_rect
        .map(|area| [area.min.x, area.min.y, area.max.x, area.max.y])
        .unwrap_or([0.0; 4]);
    TextInstanceData {
        color: [text.color.x, text.color.y, text.color.z, text.color.w],
        outline_color: [
            text.outline_color.x,
            text.outline_color.y,
            text.outline_color.z,
            text.outline_color.w,
        ],
        clip_rect: clip,
        params: [
            text.outline_width,
            layer_depth(text.layer, text.z_index),
            text.rotation,
            0.0,
        ],
        position: [text.position.x, text.position.y, text.pivot.x, text.pivot.y],
    }
}

pub fn text_signature(text: &DrawText) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.text.hash(&mut hasher);
    text.font_size.to_bits().hash(&mut hasher);
    (text.font_kind as u8).hash(&mut hasher);
    (text.alignment as u8).hash(&mut hasher);
    (text.vertical_alignment as u8).hash(&mut hasher);
    text.monospace.hash(&mut hasher);
    text.wrap_width
        .map(|width| width.to_bits())
        .unwrap_or(u32::MAX)
        .hash(&mut hasher);
    hasher.finish()
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiTextVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub character_index: u32,
    pub text_instance_index: u32,
    pub flags: u32,
}

pub const NO_TEXTURE_LAYER: u32 = 0xFFFF_FFFF;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PaintImageInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub color: [f32; 4],
    pub clip_rect: [f32; 4],
    pub texture_layer: u32,
    pub depth: f32,
    pub rotation: f32,
    pub flags: u32,
}

pub fn image_instance(image: &crate::data::DrawImage) -> PaintImageInstance {
    let clip = image
        .clip_rect
        .map(|area| [area.min.x, area.min.y, area.max.x, area.max.y])
        .unwrap_or([0.0; 4]);
    PaintImageInstance {
        position: [image.position.x, image.position.y],
        size: [image.size.x, image.size.y],
        uv_min: [image.uv_min.x, image.uv_min.y],
        uv_max: [image.uv_max.x, image.uv_max.y],
        color: [image.color.x, image.color.y, image.color.z, image.color.w],
        clip_rect: clip,
        texture_layer: image.texture_index,
        depth: layer_depth(image.layer, image.z_index),
        rotation: image.rotation,
        flags: if image.nearest {
            crate::data::IMAGE_FLAG_NEAREST
        } else {
            0
        },
    }
}
