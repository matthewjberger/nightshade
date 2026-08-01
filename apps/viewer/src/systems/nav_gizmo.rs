use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_scene::contract::GlobalTransform;
use nightshade_ui::prelude::*;

use crate::data::AXES;
use crate::resources::ViewerResources;
use crate::theme::{AXIS_COLORS, NAV_GIZMO_RADIUS, NAV_GIZMO_SIZE};

pub fn draw_nav_gizmo(
    viewer: Res<ViewerResources>,
    active_camera: Res<ActiveCamera>,
    storage: &mut nightshade::ecs::Storage,
) {
    let canvas = viewer.handles.nav_gizmo;
    if canvas == Entity::default() {
        return;
    }
    let global = active_camera
        .0
        .and_then(|camera| storage.get::<GlobalTransform>(camera))
        .copied()
        .unwrap_or_default();
    let right = global.right_vector();
    let up = global.up_vector();
    let forward = global.forward_vector();

    let center = vec2(NAV_GIZMO_SIZE * 0.5, NAV_GIZMO_SIZE * 0.5);
    let mut projected: Vec<(usize, f32, Vec2)> = AXES
        .iter()
        .enumerate()
        .map(|(index, (axis, _, _, _))| {
            let direction = Vec3::new(axis[0], axis[1], axis[2]);
            let depth = nalgebra_glm::dot(&direction, &forward);
            let point = vec2(
                center.x + nalgebra_glm::dot(&direction, &right) * NAV_GIZMO_RADIUS,
                center.y - nalgebra_glm::dot(&direction, &up) * NAV_GIZMO_RADIUS,
            );
            (index, depth, point)
        })
        .collect();
    projected.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    ui_canvas_clear(storage, canvas);
    for (index, depth, point) in projected {
        let (_, label, color_index, positive) = AXES[index];
        let fade = (1.0 - 0.28 * (depth + 1.0)).clamp(0.4, 1.0);
        let color = AXIS_COLORS[color_index];
        let faded = vec4(color.x, color.y, color.z, fade);
        if positive {
            ui_canvas_line(storage, canvas, center, point, 2.0, faded);
        }
        let radius = if positive { 9.0 } else { 6.0 };
        let fill = if positive {
            faded
        } else {
            vec4(color.x * 0.25, color.y * 0.25, color.z * 0.25, fade)
        };
        ui_canvas_circle(storage, canvas, point, radius, fill);
        ui_canvas_tag(storage, canvas, index as u32 + 1);
        if positive {
            ui_canvas_text(
                storage,
                canvas,
                label,
                vec2(point.x - 3.0, point.y - 6.0),
                11.0,
                vec4(1.0, 1.0, 1.0, fade),
            );
        }
    }
}
