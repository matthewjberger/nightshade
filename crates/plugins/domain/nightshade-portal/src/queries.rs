use crate::contract::PortalPlane;
use crate::data;
use nightshade::prelude::{Mat4, Vec2, Vec3, Vec4};
use nightshade_camera::prelude::ProjectionOverride;
use nightshade_render::queries::projection::reverse_z_perspective;
use nightshade_render::resources::viewport::Viewport;

pub fn surface_up(normal: Vec3) -> Vec3 {
    if normal.y.abs() > 0.9 {
        Vec3::z()
    } else {
        Vec3::y()
    }
}

pub fn portal_frame(portal: &PortalPlane) -> Mat4 {
    let normal = portal.normal.normalize();
    let mut right = portal.up.normalize().cross(&normal);
    if right.norm() < data::SMALLEST_AXIS {
        right = Vec3::x();
    }
    let right = right.normalize();
    let up = normal.cross(&right).normalize();
    let mut matrix = Mat4::identity();
    for row in 0..3 {
        matrix[(row, 0)] = right[row];
        matrix[(row, 1)] = up[row];
        matrix[(row, 2)] = normal[row];
        matrix[(row, 3)] = portal.center[row];
    }
    matrix
}

pub fn portal_model(portal: &PortalPlane) -> Mat4 {
    let mut model = portal_frame(portal);
    for row in 0..3 {
        model[(row, 0)] *= portal.width;
        model[(row, 1)] *= portal.height;
    }
    model
}

fn half_turn() -> Mat4 {
    Mat4::new(
        -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    )
}

pub fn portal_transform(source: &PortalPlane, destination: &PortalPlane) -> Option<Mat4> {
    let into_source = portal_frame(source).try_inverse()?;
    Some(portal_frame(destination) * half_turn() * into_source)
}

pub fn carry_matrix(source: &PortalPlane, destination: &PortalPlane) -> Mat4 {
    portal_transform(source, destination).unwrap_or_else(Mat4::identity)
}

pub fn carried_portal(portal: &PortalPlane, carry: &Mat4) -> PortalPlane {
    let moved = carry * Vec4::new(portal.center.x, portal.center.y, portal.center.z, 1.0);
    let turned = |vector: Vec3| {
        let out = carry * Vec4::new(vector.x, vector.y, vector.z, 0.0);
        Vec3::new(out.x, out.y, out.z)
    };
    PortalPlane {
        center: Vec3::new(moved.x, moved.y, moved.z),
        normal: turned(portal.normal),
        up: turned(portal.up),
        width: portal.width,
        height: portal.height,
    }
}

pub fn virtual_camera(source: &PortalPlane, destination: &PortalPlane, viewer: &Mat4) -> Mat4 {
    match portal_transform(source, destination) {
        Some(carry) => carry * viewer,
        None => *viewer,
    }
}

pub fn portal_projection(
    camera_world: &Mat4,
    destination: &PortalPlane,
    aspect: f32,
    field_of_view: f32,
    near_plane: f32,
    far_plane: f32,
    plane_bias: f32,
) -> ProjectionOverride {
    let base = reverse_z_perspective(field_of_view, aspect, near_plane, far_plane);
    let normal = destination.normal.normalize();
    let offset = -normal.dot(&destination.center) - plane_bias;
    let world_plane = Vec4::new(normal.x, normal.y, normal.z, offset);
    let view_plane = camera_world.transpose() * world_plane;

    let mut matrix = base;
    if let Some(scale) = far_corner_scale(&base, &view_plane) {
        matrix[(2, 0)] = scale * view_plane.x;
        matrix[(2, 1)] = scale * view_plane.y;
        matrix[(2, 2)] = -1.0 + scale * view_plane.z;
        matrix[(2, 3)] = scale * view_plane.w;
    }

    ProjectionOverride {
        matrix,
        z_near: near_plane,
        z_far: far_plane,
        y_fov_rad: field_of_view,
        aspect,
    }
}

fn far_corner_scale(base: &Mat4, view_plane: &Vec4) -> Option<f32> {
    let inverse = base.try_inverse()?;
    let corner = Vec4::new(
        if view_plane.x >= 0.0 { 1.0 } else { -1.0 },
        if view_plane.y >= 0.0 { 1.0 } else { -1.0 },
        0.0,
        1.0,
    );
    let found = inverse * corner;
    if found.w.abs() < 1.0e-6 {
        return None;
    }
    let point = found / found.w;
    let towards =
        view_plane.x * point.x + view_plane.y * point.y + view_plane.z * point.z + view_plane.w;
    if towards.abs() < 1.0e-6 {
        return None;
    }
    Some(point.z / towards)
}

pub fn touching_portal(center: Vec3, reach: f32, margin: f32, portal: &PortalPlane) -> bool {
    let normal = portal.normal.normalize();
    let standoff = normal.dot(&(center - portal.center));
    if standoff < 0.0 || standoff - reach > margin {
        return false;
    }

    let frame = portal_frame(portal);
    let right = Vec3::new(frame[(0, 0)], frame[(1, 0)], frame[(2, 0)]);
    let up = Vec3::new(frame[(0, 1)], frame[(1, 1)], frame[(2, 1)]);
    let flat = center - normal * standoff - portal.center;
    let across = right.dot(&flat) / (portal.width * 0.5);
    let along = up.dot(&flat) / (portal.height * 0.5);
    across * across + along * along <= 1.0
}

pub fn rim_point(portal: &PortalPlane, turn: f32, stand_off: f32, outward: f32) -> (Vec3, Vec3) {
    let frame = portal_frame(portal);
    let right = Vec3::new(frame[(0, 0)], frame[(1, 0)], frame[(2, 0)]);
    let up = Vec3::new(frame[(0, 1)], frame[(1, 1)], frame[(2, 1)]);
    let normal = Vec3::new(frame[(0, 2)], frame[(1, 2)], frame[(2, 2)]);
    let half_width = (portal.width * 0.5).max(1.0e-3);
    let half_height = (portal.height * 0.5).max(1.0e-3);
    let (sine, cosine) = (turn * std::f32::consts::TAU).sin_cos();
    let position =
        portal.center + right * cosine * half_width + up * sine * half_height + normal * stand_off;
    let away = Vec2::new(cosine / half_width, sine / half_height).normalize();
    let direction = (right * away.x + up * away.y + normal * outward).normalize();
    (position, direction)
}

pub fn position_of(matrix: &Mat4) -> Vec3 {
    Vec3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)])
}

pub fn portal_is_seen(portal: &PortalPlane, standpoint: Vec3, max_distance: f32) -> bool {
    let reach = standpoint - portal.center;
    if portal.normal.normalize().dot(&reach) <= 0.0 {
        return false;
    }
    max_distance <= 0.0 || reach.norm() <= max_distance
}

pub fn portal_target(size: (u32, u32), max: u32) -> (u32, u32) {
    let widest = size.0.max(size.1).max(1);
    if max == 0 || widest <= max {
        return size;
    }
    let share = max as f32 / widest as f32;
    (
        ((size.0 as f32 * share).round() as u32).max(1),
        ((size.1 as f32 * share).round() as u32).max(1),
    )
}

pub fn viewport_size(viewport: &Viewport) -> (u32, u32) {
    viewport
        .active_viewport_rect
        .filter(|rect| rect.width >= 1.0 && rect.height >= 1.0)
        .map(|rect| (rect.width.round() as u32, rect.height.round() as u32))
        .unwrap_or((data::FALLBACK_WIDTH, data::FALLBACK_HEIGHT))
}
