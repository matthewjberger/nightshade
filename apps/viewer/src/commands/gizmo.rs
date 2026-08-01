use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_picking::contract::PickingRay;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_scene::contract::primitives::{Name, Visibility};
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

use crate::commands::history::push;
use crate::data::{GizmoHandle, GizmoMode, PLANE_AXES, RING_RADIUS};
use crate::queries::gizmo::{
    Basis, drag_plane, grow_factor, handle_shape, intersect, local_axis, signed_angle, snapped,
    to_parent_point, to_parent_space,
};
use crate::queries::selection::snapshot;
use crate::resources::{DragStart, Edit, Gizmo, GizmoDrag, TransformEdit, ViewerResources};

pub fn start_drag(
    viewer: &mut ViewerResources,
    storage: &nightshade::ecs::Storage,
    group: &[Entity],
    basis: &Basis,
    ray: &PickingRay,
    handle: GizmoHandle,
) {
    let normal = drag_plane(viewer.gizmo.mode, basis, ray, handle);
    let Some(point) = intersect(ray, basis.origin, normal) else {
        return;
    };
    let starts: Vec<DragStart> = group
        .iter()
        .filter_map(|entity| {
            let transform = storage.get::<LocalTransform>(*entity)?;
            let global = storage.get::<GlobalTransform>(*entity)?;
            Some(DragStart {
                entity: *entity,
                transform: snapshot(transform),
                position: global.translation(),
                frame: global.0,
            })
        })
        .collect();
    if starts.is_empty() {
        return;
    }
    viewer.gizmo.drag = Some(GizmoDrag {
        handle,
        normal,
        start_point: point,
        origin: basis.origin,
        starts,
    });
}

pub fn end_drag(viewer: &mut ViewerResources, storage: &nightshade::ecs::Storage) {
    let Some(drag) = viewer.gizmo.drag.take() else {
        return;
    };
    let edits: Vec<TransformEdit> = drag
        .starts
        .iter()
        .filter_map(|start| {
            let after = snapshot(storage.get::<LocalTransform>(start.entity)?);
            (after != start.transform).then_some(TransformEdit {
                entity: start.entity,
                before: start.transform,
                after,
            })
        })
        .collect();
    if !edits.is_empty() {
        push(&mut viewer.history, Edit::Transforms(edits));
    }
}

pub fn continue_drag(
    viewer: &mut ViewerResources,
    storage: &mut nightshade::ecs::Storage,
    basis: &Basis,
    ray: &PickingRay,
    size: f32,
    snap: bool,
) {
    let Some(drag) = viewer.gizmo.drag.as_ref().cloned() else {
        return;
    };
    let Some(point) = intersect(ray, basis.origin, drag.normal) else {
        return;
    };
    let grouped = drag.starts.len() > 1;

    match viewer.gizmo.mode {
        GizmoMode::Move => {
            let world = match drag.handle {
                GizmoHandle::Axis(index) => {
                    let axis = basis.axes[index];
                    let along = snapped((point - drag.start_point).dot(&axis), SNAP_DISTANCE, snap);
                    axis * along
                }
                GizmoHandle::Plane(_) | GizmoHandle::Center => {
                    let delta = point - drag.start_point;
                    if snap {
                        Vec3::new(
                            snapped(delta.x, SNAP_DISTANCE, true),
                            snapped(delta.y, SNAP_DISTANCE, true),
                            snapped(delta.z, SNAP_DISTANCE, true),
                        )
                    } else {
                        delta
                    }
                }
            };
            for start in &drag.starts {
                let local = to_parent_space(storage, start.entity, world);
                if let Some(transform) = storage.get_mut::<LocalTransform>(start.entity) {
                    transform.translation = start.transform.translation + local;
                }
            }
        }
        GizmoMode::Rotate => {
            let GizmoHandle::Axis(index) = drag.handle else {
                return;
            };
            let from = drag.start_point - drag.origin;
            let to = point - drag.origin;
            let normal = basis.axes[index];
            let angle = snapped(
                signed_angle(from, to, normal).to_degrees(),
                SNAP_ANGLE,
                snap,
            )
            .to_radians();
            let world_step = nalgebra_glm::quat_angle_axis(angle, &normal);
            for start in &drag.starts {
                let step = nalgebra_glm::quat_angle_axis(angle, &local_axis(&start.frame, normal));
                let position = grouped.then(|| {
                    let offset = nalgebra_glm::quat_rotate_vec3(
                        &world_step,
                        &(start.position - drag.origin),
                    );
                    to_parent_point(storage, start.entity, drag.origin + offset)
                });
                if let Some(transform) = storage.get_mut::<LocalTransform>(start.entity) {
                    transform.rotation = start.transform.rotation * step;
                    if let Some(position) = position {
                        transform.translation = position;
                    }
                }
            }
        }
        GizmoMode::Scale => {
            let reach = (size * RING_RADIUS).max(1.0e-4);
            let factors = match drag.handle {
                GizmoHandle::Axis(index) => {
                    let axis = basis.axes[index];
                    let step = (point - drag.start_point).dot(&axis) / reach;
                    let factor = snapped(1.0 + step, SNAP_SCALE, snap).max(1.0e-3);
                    let mut factors = [1.0; 3];
                    factors[index] = factor;
                    factors
                }
                GizmoHandle::Plane(index) => {
                    let (first, second) = PLANE_AXES[index];
                    let factor =
                        snapped(grow_factor(&drag, point, reach), SNAP_SCALE, snap).max(1.0e-3);
                    let mut factors = [1.0; 3];
                    factors[first] = factor;
                    factors[second] = factor;
                    factors
                }
                GizmoHandle::Center => {
                    let factor =
                        snapped(grow_factor(&drag, point, reach), SNAP_SCALE, snap).max(1.0e-3);
                    [factor; 3]
                }
            };
            for start in &drag.starts {
                let position = grouped.then(|| {
                    let offset = start.position - drag.origin;
                    let mut scaled = offset;
                    for (index, factor) in factors.iter().enumerate() {
                        let axis = basis.axes[index];
                        scaled += axis * (offset.dot(&axis) * (factor - 1.0));
                    }
                    to_parent_point(storage, start.entity, drag.origin + scaled)
                });
                if let Some(transform) = storage.get_mut::<LocalTransform>(start.entity) {
                    transform.scale = Vec3::new(
                        (start.transform.scale.x * factors[0]).max(1.0e-3),
                        (start.transform.scale.y * factors[1]).max(1.0e-3),
                        (start.transform.scale.z * factors[2]).max(1.0e-3),
                    );
                    if let Some(position) = position {
                        transform.translation = position;
                    }
                }
            }
        }
    }
}

const SNAP_DISTANCE: f32 = 0.25;

const SNAP_ANGLE: f32 = 15.0;

const SNAP_SCALE: f32 = 0.1;

pub fn claim(
    capture: &mut nightshade_platform::contract::input_capture::InputCapture,
    claimed: bool,
) {
    nightshade_platform::commands::input_capture::set_pointer_claim::<Gizmo>(capture, claimed);
}

pub fn hide_handles(viewer: &mut ViewerResources, storage: &mut nightshade::ecs::Storage) {
    viewer.gizmo.hovered = None;
    viewer.gizmo.drag = None;
    let Some(entity) = viewer.gizmo.entity else {
        return;
    };
    if let Some(visibility) = storage.get_mut::<Visibility>(entity)
        && visibility.visible
    {
        visibility.visible = false;
        if let Some(lines) = storage.get_mut::<Lines>(entity) {
            lines.clear();
        }
    }
}

pub fn draw_handles(
    viewer: &mut ViewerResources,
    commands: &mut Commands,
    storage: &mut nightshade::ecs::Storage,
    basis: &Basis,
    size: f32,
) {
    let entity = match viewer.gizmo.entity {
        Some(entity) if storage.has::<Lines>(entity) => entity,
        _ => {
            let entity = commands.spawn((
                Name("Transform Handles".to_string()),
                Lines::new_always_on_top(Vec::new()),
                Visibility { visible: true },
                GlobalTransform::default(),
            ));
            viewer.gizmo.entity = Some(entity);
            return;
        }
    };
    if let Some(visibility) = storage.get_mut::<Visibility>(entity) {
        visibility.visible = true;
    }

    let active = viewer
        .gizmo
        .drag
        .as_ref()
        .map(|drag| drag.handle)
        .or(viewer.gizmo.hovered);
    let mode = viewer.gizmo.mode;
    let segments = handle_shape(mode, basis, size, active);

    if let Some(lines) = storage.get_mut::<Lines>(entity) {
        lines.clear();
        for (start, end, color) in segments {
            lines.push(Line { start, end, color });
        }
    }
}
