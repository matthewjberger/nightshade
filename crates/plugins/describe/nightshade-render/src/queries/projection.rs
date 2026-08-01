use crate::data::light_gpu::NUM_SHADOW_CASCADES;
use crate::data::shadow::{CASCADE_REFERENCE_NEAR, CASCADE_SPLIT_DISTANCES};

pub fn transform_translation(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)])
}

pub fn transform_right(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)])
}

pub fn transform_up(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)])
}

pub fn transform_forward(matrix: &nalgebra_glm::Mat4) -> nalgebra_glm::Vec3 {
    nalgebra_glm::vec3(-matrix[(0, 2)], -matrix[(1, 2)], -matrix[(2, 2)])
}

pub fn reverse_z_ortho_light(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    farthest: f32,
    nearest: f32,
) -> nalgebra_glm::Mat4 {
    let width = right - left;
    let height = top - bottom;
    let depth = nearest - farthest;

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
        -farthest / depth,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

const LIGHT_STANDS_OFF: f32 = 100.0;

pub fn cascade_view_projection_without_a_camera(
    light_direction: &nalgebra_glm::Vec3,
    cascade_far: f32,
) -> nalgebra_glm::Mat4 {
    let up = if light_direction.y.abs() > 0.99 {
        nalgebra_glm::vec3(1.0, 0.0, 0.0)
    } else {
        nalgebra_glm::vec3(0.0, 1.0, 0.0)
    };
    let middle = nalgebra_glm::vec3(0.0, 0.0, 0.0);
    let light_position = middle - light_direction * LIGHT_STANDS_OFF;
    let light_view = nalgebra_glm::look_at(&light_position, &middle, &up);
    let half = cascade_far * 0.5;
    let light_projection = reverse_z_ortho_light(
        -half,
        half,
        -half,
        half,
        -LIGHT_STANDS_OFF - cascade_far,
        -LIGHT_STANDS_OFF + cascade_far,
    );
    light_projection * light_view
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

pub fn get_orthographic_frustum_corners_world_space(
    view: &nalgebra_glm::Mat4,
    half_width: f32,
    half_height: f32,
    near: f32,
    far: f32,
) -> [nalgebra_glm::Vec3; 8] {
    let inv_view = nalgebra_glm::inverse(view);

    let corners_view = [
        nalgebra_glm::vec3(-half_width, -half_height, -near),
        nalgebra_glm::vec3(half_width, -half_height, -near),
        nalgebra_glm::vec3(half_width, half_height, -near),
        nalgebra_glm::vec3(-half_width, half_height, -near),
        nalgebra_glm::vec3(-half_width, -half_height, -far),
        nalgebra_glm::vec3(half_width, -half_height, -far),
        nalgebra_glm::vec3(half_width, half_height, -far),
        nalgebra_glm::vec3(-half_width, half_height, -far),
    ];

    let mut corners_world = [nalgebra_glm::vec3(0.0, 0.0, 0.0); 8];
    for (index, corner) in corners_view.iter().enumerate() {
        let world_pos = inv_view * nalgebra_glm::vec4(corner.x, corner.y, corner.z, 1.0);
        corners_world[index] = nalgebra_glm::vec3(world_pos.x, world_pos.y, world_pos.z);
    }
    corners_world
}

pub struct CascadeViewProjectionResult {
    pub view_projection: nalgebra_glm::Mat4,
    pub cascade_diameter: f32,
}

pub fn calculate_cascade_view_projection(
    frustum_corners: &[nalgebra_glm::Vec3; 8],
    light_direction: &nalgebra_glm::Vec3,
    cascade_resolution: f32,
) -> CascadeViewProjectionResult {
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

    let span = (max_z - min_z).max(1.0);
    let reach = radius.max(span);
    let nearest = max_z + reach;
    let farthest = min_z - span * 0.05;

    let light_projection = reverse_z_ortho_light(min_x, max_x, min_y, max_y, farthest, nearest);

    let cascade_diameter = radius * 2.0;

    CascadeViewProjectionResult {
        view_projection: light_projection * light_view,
        cascade_diameter,
    }
}

pub fn sphere_in_frustum(
    center: &nalgebra_glm::Vec3,
    radius: f32,
    frustum_planes: &[nalgebra_glm::Vec4; 6],
) -> bool {
    for plane in frustum_planes {
        let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
        if distance < -radius {
            return false;
        }
    }
    true
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

pub fn full_frustum_planes(view_projection: &[[f32; 4]; 4]) -> [nalgebra_glm::Vec4; 6] {
    let row = |index: usize| {
        [
            view_projection[0][index],
            view_projection[1][index],
            view_projection[2][index],
            view_projection[3][index],
        ]
    };
    let row_x = row(0);
    let row_y = row(1);
    let row_z = row(2);
    let row_w = row(3);
    let combine = |sign: f32, axis: [f32; 4]| {
        let plane = [
            row_w[0] + sign * axis[0],
            row_w[1] + sign * axis[1],
            row_w[2] + sign * axis[2],
            row_w[3] + sign * axis[3],
        ];
        let length = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2])
            .sqrt()
            .max(1e-8);
        nalgebra_glm::vec4(
            plane[0] / length,
            plane[1] / length,
            plane[2] / length,
            plane[3] / length,
        )
    };
    [
        combine(1.0, row_x),
        combine(-1.0, row_x),
        combine(1.0, row_y),
        combine(-1.0, row_y),
        combine(1.0, row_z),
        combine(-1.0, row_z),
    ]
}

pub fn scale_cascade_splits(camera_near: f32) -> [f32; NUM_SHADOW_CASCADES] {
    let scale = (camera_near / CASCADE_REFERENCE_NEAR).clamp(1e-6, 1.0);
    let mut splits = CASCADE_SPLIT_DISTANCES;
    for split in &mut splits {
        *split *= scale;
    }
    splits
}

pub struct CascadeFit {
    pub view_projections: [[[f32; 4]; 4]; NUM_SHADOW_CASCADES],
    pub diameters: [f32; NUM_SHADOW_CASCADES],
    pub split_distances: [f32; NUM_SHADOW_CASCADES],
}

pub fn fit_cascades(
    view: &nalgebra_glm::Mat4,
    projection: &crate::data::config::CameraProjectionParams,
    light_direction: &nalgebra_glm::Vec3,
    cascade_resolution: f32,
) -> CascadeFit {
    let camera_near = if projection.orthographic.is_some() {
        projection.z_near.max(CASCADE_REFERENCE_NEAR)
    } else {
        projection.z_near
    };
    let split_distances = scale_cascade_splits(camera_near);
    let mut view_projections = [[[0.0f32; 4]; 4]; NUM_SHADOW_CASCADES];
    let mut diameters = [0.0f32; NUM_SHADOW_CASCADES];

    for cascade_index in 0..NUM_SHADOW_CASCADES {
        let cascade_near = if cascade_index == 0 {
            camera_near
        } else {
            split_distances[cascade_index - 1]
        };
        let cascade_far = split_distances[cascade_index];

        let frustum_corners = match projection.orthographic {
            Some((half_width, half_height)) => get_orthographic_frustum_corners_world_space(
                view,
                half_width,
                half_height,
                cascade_near,
                cascade_far.min(projection.z_far),
            ),
            None => get_frustum_corners_world_space(
                view,
                projection.y_fov_rad,
                projection.aspect,
                cascade_near,
                cascade_far,
            ),
        };

        let result = calculate_cascade_view_projection(
            &frustum_corners,
            light_direction,
            cascade_resolution,
        );
        view_projections[cascade_index] = result.view_projection.into();
        diameters[cascade_index] = result.cascade_diameter;
    }

    CascadeFit {
        view_projections,
        diameters,
        split_distances,
    }
}

pub fn matrix_columns(matrix: &nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let mut columns = [[0.0f32; 4]; 4];
    for (index, column) in columns.iter_mut().enumerate() {
        for (row, value) in column.iter_mut().enumerate() {
            *value = matrix[(row, index)];
        }
    }
    columns
}
