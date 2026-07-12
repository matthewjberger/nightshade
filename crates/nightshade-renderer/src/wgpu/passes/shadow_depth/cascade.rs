pub fn reverse_z_ortho_light(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> nalgebra_glm::Mat4 {
    let width = right - left;
    let height = top - bottom;
    let depth = far - near;

    nalgebra_glm::Mat4::new(
        2.0 / width,
        0.0,
        0.0,
        -(right + left) / width,
        0.0,
        2.0 / height,
        0.0,
        -(top + bottom) / height,
        0.0,
        0.0,
        1.0 / depth,
        -near / depth,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

pub fn reverse_z_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> nalgebra_glm::Mat4 {
    let f = 1.0 / (fov / 2.0).tan();
    let depth = far - near;

    nalgebra_glm::Mat4::new(
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        near / depth,
        near * far / depth,
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}

pub fn get_frustum_corners_world_space(
    view: &nalgebra_glm::Mat4,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> [nalgebra_glm::Vec3; 8] {
    let inv_view = nalgebra_glm::inverse(view);
    let tan_half_fov = (fov / 2.0).tan();

    let near_height = near * tan_half_fov;
    let near_width = near_height * aspect;
    let far_height = far * tan_half_fov;
    let far_width = far_height * aspect;

    let corners_view = [
        nalgebra_glm::vec3(-near_width, -near_height, -near),
        nalgebra_glm::vec3(near_width, -near_height, -near),
        nalgebra_glm::vec3(near_width, near_height, -near),
        nalgebra_glm::vec3(-near_width, near_height, -near),
        nalgebra_glm::vec3(-far_width, -far_height, -far),
        nalgebra_glm::vec3(far_width, -far_height, -far),
        nalgebra_glm::vec3(far_width, far_height, -far),
        nalgebra_glm::vec3(-far_width, far_height, -far),
    ];

    let mut corners_world = [nalgebra_glm::vec3(0.0, 0.0, 0.0); 8];
    for (index, corner) in corners_view.iter().enumerate() {
        let world_pos = inv_view * nalgebra_glm::vec4(corner.x, corner.y, corner.z, 1.0);
        corners_world[index] = nalgebra_glm::vec3(world_pos.x, world_pos.y, world_pos.z);
    }
    corners_world
}

/// Builds a stable, texel-snapped cascade view-projection. The ortho box is
/// fit to the slice's bounding sphere whose radius depends only on the fixed
/// near/far/fov of the cascade, so it stays a constant size as the camera
/// orbits. The light view is anchored at the world origin (orientation only)
/// and the sphere center is snapped to the shadow-map texel grid, so shadow
/// texels never crawl or shimmer when the camera moves.
pub fn calculate_cascade_view_projection(
    frustum_corners: &[nalgebra_glm::Vec3; 8],
    light_direction: &nalgebra_glm::Vec3,
    cascade_resolution: f32,
    _cascade_far: f32,
) -> nalgebra_glm::Mat4 {
    let mut center = nalgebra_glm::vec3(0.0, 0.0, 0.0);
    for corner in frustum_corners {
        center += corner;
    }
    center /= 8.0;

    let mut radius = 0.0f32;
    for corner in frustum_corners {
        radius = radius.max(nalgebra_glm::length(&(corner - center)));
    }

    let up = if light_direction.y.abs() > 0.99 {
        nalgebra_glm::vec3(1.0, 0.0, 0.0)
    } else {
        nalgebra_glm::vec3(0.0, 1.0, 0.0)
    };

    let light_dir_normalized = light_direction.normalize();

    let light_view = nalgebra_glm::look_at(
        &(-light_dir_normalized),
        &nalgebra_glm::vec3(0.0, 0.0, 0.0),
        &up,
    );

    let center_light_space = light_view * nalgebra_glm::vec4(center.x, center.y, center.z, 1.0);

    let units_per_texel = (2.0 * radius) / cascade_resolution;
    let snapped_x = (center_light_space.x / units_per_texel).floor() * units_per_texel;
    let snapped_y = (center_light_space.y / units_per_texel).floor() * units_per_texel;

    let min_x = snapped_x - radius;
    let max_x = snapped_x + radius;
    let min_y = snapped_y - radius;
    let max_y = snapped_y + radius;

    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    for corner in frustum_corners {
        let light_space = light_view * nalgebra_glm::vec4(corner.x, corner.y, corner.z, 1.0);
        min_z = min_z.min(light_space.z);
        max_z = max_z.max(light_space.z);
    }

    let z_mult = 10.0;
    if min_z < 0.0 {
        min_z *= z_mult;
    } else {
        min_z /= z_mult;
    }
    if max_z < 0.0 {
        max_z /= z_mult;
    } else {
        max_z *= z_mult;
    }

    let light_projection = reverse_z_ortho_light(min_x, max_x, min_y, max_y, min_z, max_z);

    light_projection * light_view
}
