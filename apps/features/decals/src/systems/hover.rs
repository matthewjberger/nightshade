use crate::resources::DecalsResources;
use crate::systems::geometry::decal_preview_lines;
use nightshade::prelude::*;
use nightshade_graphics::commands::gpu_picking::request_pick_of;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_platform::prelude::*;
use nightshade_render::components::lines::Lines;
use nightshade_render::resources::gpu_picking::GpuPicking;
use nightshade_render::resources::viewport::Viewport;

pub fn track(
    mut decals: ResMut<DecalsResources>,
    input: Res<Input>,
    capture: Res<InputCapture>,
    mut gpu_picking: ResMut<GpuPicking>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    settings: Res<RenderSettings>,
) {
    let decals = &mut *decals;
    if let Some(result) = gpu_picking.take_result() {
        decals.last_pick_result = Some(result);
    }

    let cursor = input.mouse.position;
    let cursor_pixel = (cursor.x as u32, cursor.y as u32);
    if pointer_captured(&capture) || cursor_pixel == decals.last_mouse_position {
        return;
    }
    request_pick_of(
        viewport.active_viewport_rect,
        window.cached_viewport_size,
        settings.render_scale,
        &mut gpu_picking,
        cursor,
    );
    decals.last_mouse_position = cursor_pixel;
}

pub fn update_preview(decals: Res<DecalsResources>, mut previews: Query<(&mut Lines,)>) {
    let Some(preview) = decals.preview_lines_entity else {
        return;
    };
    let lines = match (&decals.last_pick_result, decals.show_preview) {
        (Some(pick), true) => {
            decal_preview_lines(pick.world_position, pick.world_normal, decals.decal_size)
        }
        _ => Lines::new(Vec::new()),
    };
    previews.for_entity(preview, |(target,)| *target = lines);
}
