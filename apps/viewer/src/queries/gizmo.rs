use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_camera::resources::ActiveCamera;
use nightshade_picking::contract::PickingRay;
use nightshade_picking::queries::{CameraView, ScreenArea};
use nightshade_platform::contract::window::Window;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::contract::GlobalTransform;

use crate::data::{
    CENTER_SIZE, GizmoHandle, GizmoMode, PICK_RADIUS, PLANE_AXES, PLANE_OFFSET, PLANE_SIZE,
    RING_RADIUS,
};
use crate::queries::lines::ring;
use crate::resources::GizmoDrag;
use crate::theme::AXIS_COLORS;

pub struct Basis {
    pub origin: Vec3,
    pub axes: [Vec3; 3],
}

pub fn group_origin(storage: &nightshade::ecs::Storage, group: &[Entity]) -> Option<Vec3> {
    if group.len() < 2 {
        return None;
    }
    let positions: Vec<Vec3> = group
        .iter()
        .filter_map(|entity| storage.get::<GlobalTransform>(*entity))
        .map(|global| global.translation())
        .collect();
    if positions.is_empty() {
        return None;
    }
    Some(positions.iter().sum::<Vec3>() / positions.len() as f32)
}

pub fn local_axis(frame: &Mat4, world_axis: Vec3) -> Vec3 {
    let Some(inverse) = frame.try_inverse() else {
        return world_axis;
    };
    let moved = inverse * nalgebra_glm::vec4(world_axis.x, world_axis.y, world_axis.z, 0.0);
    let axis = Vec3::new(moved.x, moved.y, moved.z);
    if axis.norm() < 1.0e-6 {
        world_axis
    } else {
        axis.normalize()
    }
}

pub fn basis_of(storage: &nightshade::ecs::Storage, target: Entity) -> Option<Basis> {
    let global = storage.get::<GlobalTransform>(target)?.0;
    let origin = global.column(3).xyz();
    let mut axes = [Vec3::zeros(); 3];
    for (index, axis) in axes.iter_mut().enumerate() {
        let column = global.column(index).xyz();
        if column.norm() < 1.0e-6 {
            return None;
        }
        *axis = column.normalize();
    }
    Some(Basis { origin, axes })
}

pub fn pointer_ray(
    window: &Window,
    viewport: &Viewport,
    active_camera: &ActiveCamera,
    storage: &nightshade::ecs::Storage,
    position: Vec2,
) -> Option<PickingRay> {
    let area = ScreenArea::new(window, viewport)?;
    let entity = active_camera.0?;
    let camera = storage.get::<Camera>(entity)?;
    let global = storage.get::<GlobalTransform>(entity)?;
    PickingRay::from_screen_position(area, CameraView::new(camera, global), position)
}

pub fn grow_factor(drag: &GizmoDrag, point: Vec3, reach: f32) -> f32 {
    let step = (point - drag.start_point).norm() / reach;
    let sign = if (point - drag.origin).norm() > (drag.start_point - drag.origin).norm() {
        1.0
    } else {
        -1.0
    };
    (1.0 + sign * step).max(1.0e-3)
}

pub fn snapped(value: f32, step: f32, snap: bool) -> f32 {
    if snap {
        (value / step).round() * step
    } else {
        value
    }
}

pub fn to_parent_point(storage: &nightshade::ecs::Storage, target: Entity, world: Vec3) -> Vec3 {
    let Some(parent) = storage
        .get::<nightshade::ecs::ChildOf>(target)
        .map(|link| link.0)
    else {
        return world;
    };
    let Some(global) = storage.get::<GlobalTransform>(parent) else {
        return world;
    };
    let Some(inverse) = global.0.try_inverse() else {
        return world;
    };
    let moved = inverse * nalgebra_glm::vec4(world.x, world.y, world.z, 1.0);
    Vec3::new(moved.x, moved.y, moved.z)
}

pub fn to_parent_space(storage: &nightshade::ecs::Storage, target: Entity, world: Vec3) -> Vec3 {
    let Some(parent) = storage
        .get::<nightshade::ecs::ChildOf>(target)
        .map(|link| link.0)
    else {
        return world;
    };
    let Some(global) = storage.get::<GlobalTransform>(parent) else {
        return world;
    };
    let Some(inverse) = global.0.try_inverse() else {
        return world;
    };
    let moved = inverse * nalgebra_glm::vec4(world.x, world.y, world.z, 0.0);
    Vec3::new(moved.x, moved.y, moved.z)
}

pub fn drag_plane(mode: GizmoMode, basis: &Basis, ray: &PickingRay, handle: GizmoHandle) -> Vec3 {
    match (mode, handle) {
        (_, GizmoHandle::Center) => -ray.direction,
        (GizmoMode::Rotate, GizmoHandle::Axis(index)) => basis.axes[index],
        (_, GizmoHandle::Plane(index)) => {
            let (first, second) = PLANE_AXES[index];
            basis.axes[first].cross(&basis.axes[second]).normalize()
        }
        (_, GizmoHandle::Axis(index)) => {
            let axis = basis.axes[index];
            let toward = -ray.direction;
            let side = axis.cross(&toward);
            if side.norm() < 1.0e-5 {
                axis
            } else {
                axis.cross(&side).normalize()
            }
        }
    }
}

pub fn intersect(ray: &PickingRay, origin: Vec3, normal: Vec3) -> Option<Vec3> {
    let denominator = normal.dot(&ray.direction);
    if denominator.abs() < 1.0e-6 {
        return None;
    }
    let distance = (origin - ray.origin).dot(&normal) / denominator;
    (distance > 0.0).then(|| ray.origin + ray.direction * distance)
}

pub fn signed_angle(from: Vec3, to: Vec3, normal: Vec3) -> f32 {
    let first = project_out(from, normal);
    let second = project_out(to, normal);
    if first.norm() < 1.0e-5 || second.norm() < 1.0e-5 {
        return 0.0;
    }
    let cosine = first.normalize().dot(&second.normalize()).clamp(-1.0, 1.0);
    let angle = cosine.acos();
    if first.cross(&second).dot(&normal) < 0.0 {
        -angle
    } else {
        angle
    }
}

fn project_out(value: Vec3, normal: Vec3) -> Vec3 {
    value - normal * value.dot(&normal)
}

pub fn hit_handle(
    mode: GizmoMode,
    basis: &Basis,
    ray: &PickingRay,
    size: f32,
) -> Option<GizmoHandle> {
    let radius = size * PICK_RADIUS;
    let mut best: Option<(f32, GizmoHandle)> = None;

    if mode != GizmoMode::Rotate
        && let Some(point) = intersect(ray, basis.origin, -ray.direction)
        && (point - basis.origin).norm() < size * CENTER_SIZE * 1.5
    {
        return Some(GizmoHandle::Center);
    }

    if mode == GizmoMode::Rotate {
        for index in 0..3 {
            let normal = basis.axes[index];
            let Some(point) = intersect(ray, basis.origin, normal) else {
                continue;
            };
            let distance = ((point - basis.origin).norm() - size * RING_RADIUS).abs();
            if distance < radius {
                let depth = (point - ray.origin).norm();
                if best.as_ref().is_none_or(|(near, _)| depth < *near) {
                    best = Some((depth, GizmoHandle::Axis(index)));
                }
            }
        }
        return best.map(|(_, handle)| handle);
    }

    for (index, (first, second)) in PLANE_AXES.into_iter().enumerate() {
        let normal = basis.axes[first].cross(&basis.axes[second]).normalize();
        let Some(point) = intersect(ray, basis.origin, normal) else {
            continue;
        };
        let offset = point - basis.origin;
        let along_first = offset.dot(&basis.axes[first]) / size;
        let along_second = offset.dot(&basis.axes[second]) / size;
        let low = PLANE_OFFSET - PLANE_SIZE * 0.5;
        let high = PLANE_OFFSET + PLANE_SIZE * 0.5;
        if (low..high).contains(&along_first) && (low..high).contains(&along_second) {
            let depth = (point - ray.origin).norm();
            if best.as_ref().is_none_or(|(near, _)| depth < *near) {
                best = Some((depth, GizmoHandle::Plane(index)));
            }
        }
    }

    for index in 0..3 {
        let axis = basis.axes[index];
        let end = basis.origin + axis * size;
        let Some((distance, depth)) = ray_to_segment(ray, basis.origin, end) else {
            continue;
        };
        if distance < radius && best.as_ref().is_none_or(|(near, _)| depth < *near) {
            best = Some((depth, GizmoHandle::Axis(index)));
        }
    }

    best.map(|(_, handle)| handle)
}

fn ray_to_segment(ray: &PickingRay, start: Vec3, end: Vec3) -> Option<(f32, f32)> {
    let segment = end - start;
    let between = start - ray.origin;
    let along_segment = segment.dot(&segment);
    let cross_term = segment.dot(&ray.direction);
    let denominator = along_segment - cross_term * cross_term;
    if denominator.abs() < 1.0e-6 {
        return None;
    }
    let segment_at =
        ((cross_term * between.dot(&ray.direction)) - between.dot(&segment)) / denominator;
    let segment_at = segment_at.clamp(0.0, 1.0);
    let on_segment = start + segment * segment_at;
    let ray_at = (on_segment - ray.origin).dot(&ray.direction);
    if ray_at <= 0.0 {
        return None;
    }
    let on_ray = ray.origin + ray.direction * ray_at;
    Some(((on_segment - on_ray).norm(), ray_at))
}

fn tint(index: usize, lit: bool) -> Vec4 {
    let color = AXIS_COLORS[index];
    if lit {
        vec4(1.0, 0.92, 0.35, 1.0)
    } else {
        vec4(color.x, color.y, color.z, 0.95)
    }
}

pub fn handle_shape(
    mode: GizmoMode,
    basis: &Basis,
    size: f32,
    active: Option<GizmoHandle>,
) -> Vec<(Vec3, Vec3, Vec4)> {
    let mut lines = Vec::new();
    match mode {
        GizmoMode::Rotate => {
            for (index, (first, second)) in PLANE_AXES.into_iter().enumerate() {
                let lit = active == Some(GizmoHandle::Axis(index));
                lines.extend(ring(
                    basis.origin,
                    basis.axes[first],
                    basis.axes[second],
                    size * RING_RADIUS,
                    tint(index, lit),
                ));
            }
        }
        GizmoMode::Move | GizmoMode::Scale => {
            for (index, (first, second)) in PLANE_AXES.into_iter().enumerate() {
                let lit = active == Some(GizmoHandle::Axis(index));
                let axis = basis.axes[index];
                let end = basis.origin + axis * size;
                let color = tint(index, lit);
                lines.push((basis.origin, end, color));
                let side = basis.axes[first] * (size * 0.06);
                let up = basis.axes[second] * (size * 0.06);
                if mode == GizmoMode::Move {
                    let back = end - axis * (size * 0.16);
                    for corner in [side, -side, up, -up] {
                        lines.push((end, back + corner, color));
                    }
                } else {
                    let back = end - axis * (size * 0.1);
                    let corners = [
                        back + side + up,
                        back - side + up,
                        back - side - up,
                        back + side - up,
                    ];
                    for position in 0..4 {
                        lines.push((corners[position], corners[(position + 1) % 4], color));
                        lines.push((
                            corners[position],
                            corners[position] + axis * (size * 0.1),
                            color,
                        ));
                    }
                }
            }
            for (index, (first, second)) in PLANE_AXES.into_iter().enumerate() {
                let lit = active == Some(GizmoHandle::Plane(index));
                let color = tint(index, lit);
                let low = size * (PLANE_OFFSET - PLANE_SIZE * 0.5);
                let high = size * (PLANE_OFFSET + PLANE_SIZE * 0.5);
                let one = basis.axes[first];
                let two = basis.axes[second];
                let corners = [
                    basis.origin + one * low + two * low,
                    basis.origin + one * high + two * low,
                    basis.origin + one * high + two * high,
                    basis.origin + one * low + two * high,
                ];
                for position in 0..4 {
                    lines.push((corners[position], corners[(position + 1) % 4], color));
                }
            }
            let lit = active == Some(GizmoHandle::Center);
            let color = if lit {
                vec4(1.0, 0.92, 0.35, 1.0)
            } else {
                vec4(0.92, 0.92, 0.92, 0.9)
            };
            cube_edges(&mut lines, basis, size * CENTER_SIZE, color);
        }
    }
    lines
}

fn cube_edges(lines: &mut Vec<(Vec3, Vec3, Vec4)>, basis: &Basis, half: f32, color: Vec4) {
    let corner = |signs: [f32; 3]| {
        basis.origin
            + basis.axes[0] * (half * signs[0])
            + basis.axes[1] * (half * signs[1])
            + basis.axes[2] * (half * signs[2])
    };
    let corners: Vec<Vec3> = (0..8)
        .map(|index| {
            corner([
                if index & 1 == 0 { -1.0 } else { 1.0 },
                if index & 2 == 0 { -1.0 } else { 1.0 },
                if index & 4 == 0 { -1.0 } else { 1.0 },
            ])
        })
        .collect();
    for (start, end) in [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        lines.push((corners[start], corners[end], color));
    }
}
