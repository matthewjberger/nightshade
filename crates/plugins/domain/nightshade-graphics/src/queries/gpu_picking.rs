use nalgebra_glm::Vec2;

use nightshade_render::data::config::ViewportRect;

pub fn surface_pick_coords(
    mouse_pos: Vec2,
    viewport_rect: Option<ViewportRect>,
    surface_size: (u32, u32),
) -> Option<(u32, u32)> {
    let Some(rect) = viewport_rect else {
        return Some((mouse_pos.x.max(0.0) as u32, mouse_pos.y.max(0.0) as u32));
    };

    let local = rect.to_local(mouse_pos);
    if local.x < 0.0 || local.y < 0.0 || local.x >= rect.width || local.y >= rect.height {
        return None;
    }

    let surface_width = surface_size.0 as f32;
    let surface_height = surface_size.1 as f32;
    if surface_width <= 0.0 || surface_height <= 0.0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }

    let texture_aspect = surface_width / surface_height;
    let rect_aspect = rect.width / rect.height;
    let (uv_scale_x, uv_scale_y) = if texture_aspect > rect_aspect {
        (rect_aspect / texture_aspect, 1.0)
    } else if texture_aspect < rect_aspect {
        (1.0, texture_aspect / rect_aspect)
    } else {
        (1.0, 1.0)
    };

    let tile_uv_x = local.x / rect.width;
    let tile_uv_y = local.y / rect.height;
    let input_uv_x = (tile_uv_x - 0.5) * uv_scale_x + 0.5;
    let input_uv_y = (tile_uv_y - 0.5) * uv_scale_y + 0.5;

    let pick_x = (input_uv_x * surface_width)
        .clamp(0.0, surface_width - 1.0)
        .round() as u32;
    let pick_y = (input_uv_y * surface_height)
        .clamp(0.0, surface_height - 1.0)
        .round() as u32;
    Some((pick_x, pick_y))
}
