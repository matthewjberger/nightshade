use nalgebra_glm::Vec2;

use crate::queries::gpu_picking::surface_pick_coords;
use nightshade_render::data::config::ViewportRect;
use nightshade_render::resources::gpu_picking::GpuPicking;

pub fn request_pick_at(resources: &mut nightshade_ecs::ResourceMap, cursor_position: Vec2) {
    let rect = resources
        .res::<nightshade_render::resources::viewport::Viewport>()
        .active_viewport_rect;
    let window_size = resources
        .res::<nightshade_platform::contract::window::Window>()
        .cached_viewport_size;
    let render_scale = resources
        .res::<nightshade_render::data::config::RenderSettings>()
        .render_scale;
    request_pick_of(
        rect,
        window_size,
        render_scale,
        resources.res_mut::<GpuPicking>(),
        cursor_position,
    );
}

pub fn request_pick_of(
    viewport_rect: Option<ViewportRect>,
    window_viewport_size: Option<(u32, u32)>,
    render_scale: f32,
    picking: &mut GpuPicking,
    cursor_position: Vec2,
) {
    let rect = viewport_rect.or_else(|| {
        window_viewport_size.map(|(width, height)| ViewportRect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        })
    });
    let render_scale = render_scale.clamp(0.25, 4.0);
    let Some(surface_size) = rect.map(|rect| {
        (
            ((rect.width * render_scale).round() as u32).max(1),
            ((rect.height * render_scale).round() as u32).max(1),
        )
    }) else {
        return;
    };
    if let Some((pick_x, pick_y)) = surface_pick_coords(cursor_position, rect, surface_size) {
        picking.request_pick(pick_x, pick_y);
    }
}
