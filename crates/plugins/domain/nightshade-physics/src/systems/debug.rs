use crate::components::PhysicsDebugDraw;
use nalgebra_glm::{Vec3, Vec4, vec3};
use nightshade_render::components::lines::Line;
use nightshade_scene::contract::primitives::Name;
use rapier3d::na::UnitQuaternion;
use std::f32::consts::PI;

use crate::prelude::*;
const COLLIDER_COLOR: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);
const DYNAMIC_BODY_COLOR: Vec4 = Vec4::new(0.0, 0.5, 1.0, 1.0);
const KINEMATIC_BODY_COLOR: Vec4 = Vec4::new(1.0, 0.5, 0.0, 1.0);
const STATIC_BODY_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);
const JOINT_COLOR: Vec4 = Vec4::new(1.0, 1.0, 0.0, 1.0);
const JOINT_ANCHOR_COLOR: Vec4 = Vec4::new(1.0, 0.2, 0.8, 1.0);
const CIRCLE_SEGMENTS: usize = 24;

pub fn physics_debug_draw_system(
    mut physics: ResMut<PhysicsWorld>,
    mut commands: Commands,
    target: Query<
        (
            &mut nightshade_render::components::lines::Lines,
            &mut nightshade_scene::contract::primitives::Visibility,
        ),
        With<PhysicsDebugDraw>,
    >,
) {
    let mut found = false;
    target.for_each(|_entity, (lines_component, visibility)| {
        found = true;
        visibility.visible = physics.debug_draw;
        if !physics.debug_draw {
            return;
        }
        if let Some(lines) = build_debug_lines(&mut physics) {
            lines_component.lines = lines;
            lines_component.mark_dirty();
        }
    });

    if !found && physics.debug_draw {
        let entity = commands.spawn((
            nightshade_render::components::lines::Lines::new_always_on_top(Vec::new()),
            nightshade_scene::contract::primitives::Visibility::default(),
            nightshade_scene::contract::LocalTransform::default(),
            nightshade_scene::contract::GlobalTransform::default(),
            Name("Physics Debug Gizmo".to_string()),
        ));
        commands.add_tag_type::<PhysicsDebugDraw>(entity);
    }
}

fn build_debug_lines(physics: &mut PhysicsWorld) -> Option<Vec<Line>> {
    let current_version = physics.debug_version;
    let version_changed = current_version != physics.debug_last_version;

    if version_changed {
        physics.debug_static_lines_cache.clear();
        physics.debug_last_version = current_version;
    }

    let mut has_dynamic_or_kinematic = false;
    let mut all_static_cached = !version_changed;

    if all_static_cached {
        for (collider_handle, collider) in physics.collider_set.iter() {
            let parent_handle = collider.parent();
            let body_type = parent_handle
                .and_then(|h| physics.rigid_body_set.get(h))
                .map(|rb| rb.body_type());

            let is_static = matches!(body_type, Some(rapier3d::prelude::RigidBodyType::Fixed));

            if !is_static {
                has_dynamic_or_kinematic = true;
                break;
            }

            if !physics
                .debug_static_lines_cache
                .contains_key(&collider_handle)
            {
                all_static_cached = false;
                break;
            }
        }
    } else {
        for (_collider_handle, collider) in physics.collider_set.iter() {
            let parent_handle = collider.parent();
            let body_type = parent_handle
                .and_then(|h| physics.rigid_body_set.get(h))
                .map(|rb| rb.body_type());

            if !matches!(body_type, Some(rapier3d::prelude::RigidBodyType::Fixed)) {
                has_dynamic_or_kinematic = true;
                break;
            }
        }
    }

    if all_static_cached && !has_dynamic_or_kinematic {
        return None;
    }

    let mut lines = Vec::new();
    let mut static_colliders_to_cache = Vec::new();

    for (collider_handle, collider) in physics.collider_set.iter() {
        let parent_handle = collider.parent();
        let body_type = parent_handle
            .and_then(|h| physics.rigid_body_set.get(h))
            .map(|rb| rb.body_type());

        let is_static = matches!(body_type, Some(rapier3d::prelude::RigidBodyType::Fixed));

        if is_static
            && let Some(cached_lines) = physics.debug_static_lines_cache.get(&collider_handle)
        {
            lines.extend_from_slice(cached_lines);
            continue;
        }

        let color = match body_type {
            Some(rapier3d::prelude::RigidBodyType::Dynamic) => DYNAMIC_BODY_COLOR,
            Some(rapier3d::prelude::RigidBodyType::KinematicPositionBased)
            | Some(rapier3d::prelude::RigidBodyType::KinematicVelocityBased) => {
                KINEMATIC_BODY_COLOR
            }
            Some(rapier3d::prelude::RigidBodyType::Fixed) => STATIC_BODY_COLOR,
            None => COLLIDER_COLOR,
        };

        let position = collider.position();
        let translation = vec3(
            position.translation.x,
            position.translation.y,
            position.translation.z,
        );
        let rotation = position.rotation;

        let start_index = lines.len();
        generate_collider_lines(&mut lines, collider.shape(), translation, rotation, color);

        if is_static {
            let collider_lines: Vec<Line> = lines[start_index..].to_vec();
            static_colliders_to_cache.push((collider_handle, collider_lines));
        }
    }

    for (handle, collider_lines) in static_colliders_to_cache {
        physics
            .debug_static_lines_cache
            .insert(handle, collider_lines);
    }

    generate_joint_lines(&mut lines, physics);

    Some(lines)
}

fn generate_joint_lines(lines: &mut Vec<Line>, physics: &PhysicsWorld) {
    for (_handle, joint) in physics.impulse_joint_set.iter() {
        let body1_handle = joint.body1;
        let body2_handle = joint.body2;

        let Some(body1) = physics.rigid_body_set.get(body1_handle) else {
            continue;
        };
        let Some(body2) = physics.rigid_body_set.get(body2_handle) else {
            continue;
        };

        let body1_pos = body1.position();
        let body2_pos = body2.position();

        let anchor1_local = joint.data.local_anchor1();
        let anchor2_local = joint.data.local_anchor2();

        let anchor1_world = body1_pos * anchor1_local;
        let anchor2_world = body2_pos * anchor2_local;

        let anchor1 = vec3(anchor1_world.x, anchor1_world.y, anchor1_world.z);
        let anchor2 = vec3(anchor2_world.x, anchor2_world.y, anchor2_world.z);

        lines.push(Line {
            start: anchor1,
            end: anchor2,
            color: JOINT_COLOR,
        });

        let body1_center = vec3(
            body1_pos.translation.x,
            body1_pos.translation.y,
            body1_pos.translation.z,
        );
        let body2_center = vec3(
            body2_pos.translation.x,
            body2_pos.translation.y,
            body2_pos.translation.z,
        );

        lines.push(Line {
            start: body1_center,
            end: anchor1,
            color: JOINT_ANCHOR_COLOR,
        });
        lines.push(Line {
            start: body2_center,
            end: anchor2,
            color: JOINT_ANCHOR_COLOR,
        });

        generate_joint_anchor_marker(lines, anchor1, JOINT_ANCHOR_COLOR);
        generate_joint_anchor_marker(lines, anchor2, JOINT_ANCHOR_COLOR);
    }
}

fn generate_joint_anchor_marker(lines: &mut Vec<Line>, position: Vec3, color: Vec4) {
    let size = 0.05;
    lines.push(Line {
        start: position - vec3(size, 0.0, 0.0),
        end: position + vec3(size, 0.0, 0.0),
        color,
    });
    lines.push(Line {
        start: position - vec3(0.0, size, 0.0),
        end: position + vec3(0.0, size, 0.0),
        color,
    });
    lines.push(Line {
        start: position - vec3(0.0, 0.0, size),
        end: position + vec3(0.0, 0.0, size),
        color,
    });
}

fn generate_collider_lines(
    lines: &mut Vec<Line>,
    shape: &dyn rapier3d::prelude::Shape,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    color: Vec4,
) {
    if let Some(ball) = shape.as_ball() {
        generate_sphere_lines(lines, translation, rotation, ball.radius, color);
    } else if let Some(cuboid) = shape.as_cuboid() {
        generate_cuboid_lines(lines, translation, rotation, cuboid.half_extents, color);
    } else if let Some(capsule) = shape.as_capsule() {
        generate_capsule_lines(
            lines,
            translation,
            rotation,
            capsule.half_height(),
            capsule.radius,
            color,
        );
    } else if let Some(cylinder) = shape.as_cylinder() {
        generate_cylinder_lines(
            lines,
            translation,
            rotation,
            cylinder.half_height,
            cylinder.radius,
            color,
        );
    } else if let Some(cone) = shape.as_cone() {
        generate_cone_lines(
            lines,
            translation,
            rotation,
            cone.half_height,
            cone.radius,
            color,
        );
    } else if let Some(trimesh) = shape.as_trimesh() {
        generate_trimesh_lines(lines, translation, rotation, trimesh, color);
    } else if let Some(convex) = shape.as_convex_polyhedron() {
        generate_convex_lines(lines, translation, rotation, convex, color);
    } else if let Some(heightfield) = shape.as_heightfield() {
        generate_heightfield_lines(lines, translation, rotation, heightfield, color);
    }
}

fn rotate_point(point: Vec3, rotation: UnitQuaternion<f32>) -> Vec3 {
    let rotated = rotation * rapier3d::prelude::Point::new(point.x, point.y, point.z);
    vec3(rotated.x, rotated.y, rotated.z)
}

fn transform_point(point: Vec3, translation: Vec3, rotation: UnitQuaternion<f32>) -> Vec3 {
    rotate_point(point, rotation) + translation
}

fn generate_sphere_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    radius: f32,
    color: Vec4,
) {
    generate_circle_lines(
        lines,
        translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        color,
    );
    generate_circle_lines(
        lines,
        translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );
    generate_circle_lines(
        lines,
        translation,
        rotation,
        radius,
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );
}

fn generate_circle_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    radius: f32,
    axis1: Vec3,
    axis2: Vec3,
    color: Vec4,
) {
    for segment_index in 0..CIRCLE_SEGMENTS {
        let angle1 = (segment_index as f32 / CIRCLE_SEGMENTS as f32) * 2.0 * PI;
        let angle2 = ((segment_index + 1) as f32 / CIRCLE_SEGMENTS as f32) * 2.0 * PI;

        let local_point1 = axis1 * angle1.cos() * radius + axis2 * angle1.sin() * radius;
        let local_point2 = axis1 * angle2.cos() * radius + axis2 * angle2.sin() * radius;

        let point1 = transform_point(local_point1, translation, rotation);
        let point2 = transform_point(local_point2, translation, rotation);

        lines.push(Line {
            start: point1,
            end: point2,
            color,
        });
    }
}

fn generate_cuboid_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    half_extents: rapier3d::prelude::Vector<f32>,
    color: Vec4,
) {
    let hx = half_extents.x;
    let hy = half_extents.y;
    let hz = half_extents.z;

    let corners = [
        vec3(-hx, -hy, -hz),
        vec3(hx, -hy, -hz),
        vec3(hx, hy, -hz),
        vec3(-hx, hy, -hz),
        vec3(-hx, -hy, hz),
        vec3(hx, -hy, hz),
        vec3(hx, hy, hz),
        vec3(-hx, hy, hz),
    ];

    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    for (start_index, end_index) in edges {
        let start = transform_point(corners[start_index], translation, rotation);
        let end = transform_point(corners[end_index], translation, rotation);
        lines.push(Line { start, end, color });
    }
}

fn generate_capsule_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    half_height: f32,
    radius: f32,
    color: Vec4,
) {
    let top_center = vec3(0.0, half_height, 0.0);
    let bottom_center = vec3(0.0, -half_height, 0.0);

    let top_translation = translation + rotate_point(top_center, rotation);
    let bottom_translation = translation + rotate_point(bottom_center, rotation);

    generate_circle_lines(
        lines,
        top_translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );
    generate_circle_lines(
        lines,
        bottom_translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );

    for segment_index in 0..4 {
        let angle = (segment_index as f32 / 4.0) * 2.0 * PI;
        let offset = vec3(angle.cos() * radius, 0.0, angle.sin() * radius);

        let top = transform_point(top_center + offset, translation, rotation);
        let bottom = transform_point(bottom_center + offset, translation, rotation);

        lines.push(Line {
            start: top,
            end: bottom,
            color,
        });
    }

    generate_hemisphere_lines(lines, top_translation, rotation, radius, true, color);
    generate_hemisphere_lines(lines, bottom_translation, rotation, radius, false, color);
}

fn generate_hemisphere_lines(
    lines: &mut Vec<Line>,
    center: Vec3,
    rotation: UnitQuaternion<f32>,
    radius: f32,
    top: bool,
    color: Vec4,
) {
    let arc_segments = CIRCLE_SEGMENTS / 4;
    let direction = if top { 1.0 } else { -1.0 };

    for arc_index in 0..2 {
        let base_angle = arc_index as f32 * PI / 2.0;
        for segment_index in 0..arc_segments {
            let phi1 = (segment_index as f32 / arc_segments as f32) * (PI / 2.0);
            let phi2 = ((segment_index + 1) as f32 / arc_segments as f32) * (PI / 2.0);

            let local_point1 = vec3(
                phi1.cos() * base_angle.cos() * radius,
                phi1.sin() * radius * direction,
                phi1.cos() * base_angle.sin() * radius,
            );
            let local_point2 = vec3(
                phi2.cos() * base_angle.cos() * radius,
                phi2.sin() * radius * direction,
                phi2.cos() * base_angle.sin() * radius,
            );

            let point1 = center + rotate_point(local_point1, rotation);
            let point2 = center + rotate_point(local_point2, rotation);

            lines.push(Line {
                start: point1,
                end: point2,
                color,
            });
        }
    }
}

fn generate_cylinder_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    half_height: f32,
    radius: f32,
    color: Vec4,
) {
    let top_center = vec3(0.0, half_height, 0.0);
    let bottom_center = vec3(0.0, -half_height, 0.0);

    let top_translation = translation + rotate_point(top_center, rotation);
    let bottom_translation = translation + rotate_point(bottom_center, rotation);

    generate_circle_lines(
        lines,
        top_translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );
    generate_circle_lines(
        lines,
        bottom_translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );

    for segment_index in 0..4 {
        let angle = (segment_index as f32 / 4.0) * 2.0 * PI;
        let offset = vec3(angle.cos() * radius, 0.0, angle.sin() * radius);

        let top = transform_point(top_center + offset, translation, rotation);
        let bottom = transform_point(bottom_center + offset, translation, rotation);

        lines.push(Line {
            start: top,
            end: bottom,
            color,
        });
    }
}

fn generate_cone_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    half_height: f32,
    radius: f32,
    color: Vec4,
) {
    let apex = vec3(0.0, half_height, 0.0);
    let base_center = vec3(0.0, -half_height, 0.0);

    let base_translation = translation + rotate_point(base_center, rotation);
    generate_circle_lines(
        lines,
        base_translation,
        rotation,
        radius,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        color,
    );

    let apex_world = transform_point(apex, translation, rotation);

    for segment_index in 0..4 {
        let angle = (segment_index as f32 / 4.0) * 2.0 * PI;
        let base_point = vec3(angle.cos() * radius, -half_height, angle.sin() * radius);
        let base_world = transform_point(base_point, translation, rotation);

        lines.push(Line {
            start: apex_world,
            end: base_world,
            color,
        });
    }
}

fn generate_trimesh_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    trimesh: &rapier3d::prelude::TriMesh,
    color: Vec4,
) {
    for triangle_index in 0..trimesh.num_triangles() {
        let triangle = trimesh.triangle(triangle_index as u32);
        let vertex_a = transform_point(
            vec3(triangle.a.x, triangle.a.y, triangle.a.z),
            translation,
            rotation,
        );
        let vertex_b = transform_point(
            vec3(triangle.b.x, triangle.b.y, triangle.b.z),
            translation,
            rotation,
        );
        let vertex_c = transform_point(
            vec3(triangle.c.x, triangle.c.y, triangle.c.z),
            translation,
            rotation,
        );

        lines.push(Line {
            start: vertex_a,
            end: vertex_b,
            color,
        });
        lines.push(Line {
            start: vertex_b,
            end: vertex_c,
            color,
        });
        lines.push(Line {
            start: vertex_c,
            end: vertex_a,
            color,
        });
    }
}

fn generate_convex_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    convex: &rapier3d::prelude::ConvexPolyhedron,
    color: Vec4,
) {
    let vertices = convex.points();

    for face_index in 0..convex.faces().len() {
        let face = &convex.faces()[face_index];
        let first_vertex_index = face.first_vertex_or_edge as usize;
        let vertex_count = face.num_vertices_or_edges as usize;

        for local_vertex_index in 0..vertex_count {
            let current_index =
                convex.vertices_adj_to_face()[first_vertex_index + local_vertex_index] as usize;
            let next_index = convex.vertices_adj_to_face()
                [first_vertex_index + (local_vertex_index + 1) % vertex_count]
                as usize;

            let current_vertex = &vertices[current_index];
            let next_vertex = &vertices[next_index];

            let start = transform_point(
                vec3(current_vertex.x, current_vertex.y, current_vertex.z),
                translation,
                rotation,
            );
            let end = transform_point(
                vec3(next_vertex.x, next_vertex.y, next_vertex.z),
                translation,
                rotation,
            );

            lines.push(Line { start, end, color });
        }
    }
}

fn generate_heightfield_lines(
    lines: &mut Vec<Line>,
    translation: Vec3,
    rotation: UnitQuaternion<f32>,
    heightfield: &rapier3d::prelude::HeightField,
    color: Vec4,
) {
    let scale = heightfield.scale();
    let (rows, cols) = (heightfield.nrows(), heightfield.ncols());
    let heights = heightfield.heights();

    for row_index in 0..rows {
        for col_index in 0..cols {
            let height = heights[(row_index, col_index)];
            let x = (col_index as f32 - cols as f32 / 2.0) * scale.x / cols as f32;
            let z = (row_index as f32 - rows as f32 / 2.0) * scale.z / rows as f32;
            let y = height * scale.y;

            let current_point = transform_point(vec3(x, y, z), translation, rotation);

            if col_index + 1 < cols {
                let next_height = heights[(row_index, col_index + 1)];
                let next_x = ((col_index + 1) as f32 - cols as f32 / 2.0) * scale.x / cols as f32;
                let next_y = next_height * scale.y;
                let next_point = transform_point(vec3(next_x, next_y, z), translation, rotation);
                lines.push(Line {
                    start: current_point,
                    end: next_point,
                    color,
                });
            }

            if row_index + 1 < rows {
                let next_height = heights[(row_index + 1, col_index)];
                let next_z = ((row_index + 1) as f32 - rows as f32 / 2.0) * scale.z / rows as f32;
                let next_y = next_height * scale.y;
                let next_point = transform_point(vec3(x, next_y, next_z), translation, rotation);
                lines.push(Line {
                    start: current_point,
                    end: next_point,
                    color,
                });
            }
        }
    }
}
