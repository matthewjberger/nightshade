use crate::components::DemoMesh;
use crate::resources::{PickingMode, PickingResources};
use crate::systems::geometry::preview_marker;
use crate::theme::PREVIEW_MARKER_SIZE;
use nightshade::prelude::*;
use nightshade_graphics::commands::gpu_picking::request_pick_of;
use nightshade_graphics::prelude::*;
use nightshade_picking::{PickingOptions, pick_entities, screen_ray};
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_render::components::lines::Lines;
use nightshade_render::resources::gpu_picking::GpuPicking;

struct PickSurface {
    rect: Option<nightshade_render::data::config::ViewportRect>,
    window_size: Option<(u32, u32)>,
    render_scale: f32,
}

struct RayView<'a> {
    window: &'a nightshade_platform::contract::window::Window,
    viewport: &'a nightshade_render::resources::viewport::Viewport,
    active_camera: &'a nightshade_camera::resources::ActiveCamera,
}

pub fn update(
    mut picking: ResMut<PickingResources>,
    input: Res<Input>,
    mut gpu_picking: ResMut<GpuPicking>,
    window: Res<nightshade_platform::contract::window::Window>,
    viewport: Res<nightshade_render::resources::viewport::Viewport>,
    settings: Res<RenderSettings>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    storage: &mut nightshade::ecs::Storage,
) {
    let picking = &mut *picking;
    match picking.mode {
        PickingMode::GpuHover => {
            let surface = PickSurface {
                rect: viewport.active_viewport_rect,
                window_size: window.cached_viewport_size,
                render_scale: settings.render_scale,
            };
            gpu_hover(picking, &input, &mut gpu_picking, &surface, storage);
        }
        PickingMode::CpuHover => {
            let view = RayView {
                window: &window,
                viewport: &viewport,
                active_camera: &active_camera,
            };
            cpu_hover(picking, &input, &view, storage);
        }
    }
}

fn gpu_hover(
    picking: &mut PickingResources,
    input: &Input,
    gpu_picking: &mut GpuPicking,
    surface: &PickSurface,
    storage: &mut nightshade::ecs::Storage,
) {
    if let Some(result) = gpu_picking.take_result() {
        picking.last_pick_result = Some(result);
    }

    let cursor = input.mouse.position;
    let cursor_pixel = (cursor.x as u32, cursor.y as u32);
    let mouse_held = input.mouse.state.contains(MouseState::LEFT_CLICKED);
    if mouse_held {
        picking.highlighted_entity = None;
    } else if cursor_pixel != picking.last_mouse_pos {
        request_pick_of(
            surface.rect,
            surface.window_size,
            surface.render_scale,
            gpu_picking,
            cursor,
        );
        picking.last_mouse_pos = cursor_pixel;
    }

    draw_marker(picking, mouse_held, storage);
    if mouse_held {
        return;
    }

    picking.highlighted_entity = picking.last_pick_result.as_ref().and_then(|result| {
        if result.depth <= 0.0 {
            return None;
        }
        let picked = result.entity_id?;
        storage
            .query_entities::<(DemoMesh,)>()
            .find(|entity| entity.id == picked)
    });
}

fn cpu_hover(
    picking: &mut PickingResources,
    input: &Input,
    view: &RayView<'_>,
    storage: &mut nightshade::ecs::Storage,
) {
    if let Some(preview_entity) = picking.preview_lines_entity {
        storage.set(preview_entity, Lines::new(vec![]));
    }
    if input.mouse.state.contains(MouseState::LEFT_CLICKED) {
        picking.highlighted_entity = None;
        return;
    }
    let Some(ray) = screen_ray(
        view.window,
        view.viewport,
        view.active_camera,
        storage.query_ref(),
        input.mouse.position,
    ) else {
        return;
    };
    let hits = pick_entities(storage.query_ref(), ray, PickingOptions::default());
    let demo_meshes: std::collections::HashSet<Entity> =
        storage.query_entities::<(DemoMesh,)>().collect();
    picking.highlighted_entity = hits
        .into_iter()
        .find(|hit| demo_meshes.contains(&hit.entity))
        .map(|hit| hit.entity);
}

fn draw_marker(
    picking: &PickingResources,
    mouse_held: bool,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(preview_entity) = picking.preview_lines_entity else {
        return;
    };
    let lines = match &picking.last_pick_result {
        Some(result) if result.depth > 0.0 && !mouse_held => preview_marker(
            result.world_position,
            result.world_normal,
            PREVIEW_MARKER_SIZE,
        ),
        _ => Lines::new(vec![]),
    };
    storage.set(preview_entity, lines);
}
