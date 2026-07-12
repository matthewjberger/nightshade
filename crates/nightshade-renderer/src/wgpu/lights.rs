//! CPU-side collection of the frame's light lists from [`RendererState`]
//! into the packed GPU structures the mesh and shadow passes consume, plus
//! the cascade shadow fit against the dispatching camera's frustum.

use crate::config::{
    RenderAreaLightShape, RenderLightData, RenderLightType, RenderLighting, RendererState,
};
use crate::wgpu::passes::geometry::projection::{
    AREA_EMISSIVE_LAYER_NONE, AREA_SHAPE_DISK, AREA_SHAPE_RECTANGLE, AREA_SHAPE_SPHERE,
    AREA_SHAPE_TUBE, AreaLightCollectionResult, AreaLightData, COOKIE_LAYER_NONE,
    CascadeShadowResult, LightCollectionResult, LightData, MAX_AREA_LIGHTS,
    calculate_cascade_view_projection, default_shadow_normal_bias_value,
    get_frustum_corners_world_space, reverse_z_ortho_light, transform_forward, transform_right,
    transform_translation, transform_up,
};
use crate::wgpu::passes::shadow_depth::{CASCADE_SPLIT_DISTANCES, scale_cascade_splits};
use crate::wgpu::render_configs::CameraFrameInputs;

pub fn collect_lights(scene: &RendererState, max_lights: usize) -> LightCollectionResult {
    let mut directional_lights = Vec::new();
    let mut local_lights = Vec::new();
    let mut directional_light = None;
    let mut directional_entities = Vec::new();
    let mut local_entities = Vec::new();

    for render_light in &scene.render_lights.lights {
        let entity = render_light.entity;
        let light = &render_light.light;
        let transform = &render_light.transform;
        {
            if light.light_type == RenderLightType::Area {
                continue;
            }

            let light_type = match light.light_type {
                RenderLightType::Directional => 0,
                RenderLightType::Point => 1,
                RenderLightType::Spot => 2,
                RenderLightType::Area => continue,
            };

            if light_type == 0 && light.cast_shadows && directional_light.is_none() {
                directional_light = Some((light.clone(), *transform));
            }

            let position = transform_translation(transform);
            let direction = transform_forward(transform);

            let light_size = match light.light_type {
                RenderLightType::Directional => 1.0,
                RenderLightType::Point => (light.range * 0.05).max(0.1),
                RenderLightType::Spot => (light.range * 0.03).max(0.1),
                RenderLightType::Area => 0.1,
            };

            let light_data = LightData {
                position: [position.x, position.y, position.z, 1.0],
                direction: [direction.x, direction.y, direction.z, 0.0],
                color: [
                    light.color.x * light.intensity,
                    light.color.y * light.intensity,
                    light.color.z * light.intensity,
                    1.0,
                ],
                light_type,
                range: light.range,
                inner_cone: light.inner_cone_angle.cos(),
                outer_cone: light.outer_cone_angle.cos(),
                shadow_index: -1,
                light_size,
                cookie_layer: COOKIE_LAYER_NONE,
                _padding: 0.0,
            };

            if light_type == 0 {
                directional_lights.push(light_data);
                directional_entities.push(entity);
            } else {
                local_lights.push(light_data);
                local_entities.push(entity);
            }

            if directional_lights.len() + local_lights.len() >= max_lights {
                break;
            }
        }
    }

    let num_directional_lights = directional_lights.len() as u32;
    let mut lights_data = directional_lights;
    lights_data.extend(local_lights);

    let mut entity_to_index = std::collections::HashMap::new();
    for (index, entity) in directional_entities.iter().enumerate() {
        entity_to_index.insert(*entity, index);
    }
    let offset = directional_entities.len();
    for (index, entity) in local_entities.iter().enumerate() {
        entity_to_index.insert(*entity, offset + index);
    }

    LightCollectionResult {
        lights_data,
        directional_light,
        num_directional_lights,
        entity_to_index,
    }
}

pub fn collect_area_lights(
    scene: &RendererState,
    max_area_lights: usize,
) -> AreaLightCollectionResult {
    let mut area_lights_data = Vec::new();
    let mut entity_to_index = std::collections::HashMap::new();

    for render_light in &scene.render_lights.lights {
        if area_lights_data.len() >= max_area_lights {
            break;
        }

        let entity = render_light.entity;
        let light = &render_light.light;
        let transform = &render_light.transform;

        if light.light_type != RenderLightType::Area {
            continue;
        }

        let center = transform_translation(transform);
        let right_dir = transform_right(transform);
        let up_dir = transform_up(transform);
        let normal = transform_forward(transform);

        let (right_half, up_half, radius) = match light.area_shape {
            RenderAreaLightShape::Rectangle => (
                right_dir * (light.area_width * 0.5),
                up_dir * (light.area_height * 0.5),
                (light.area_width + light.area_height) * 0.1,
            ),
            RenderAreaLightShape::Disk => (
                right_dir * light.area_radius,
                up_dir * light.area_radius,
                light.area_radius,
            ),
            RenderAreaLightShape::Sphere => (
                right_dir * light.area_radius,
                up_dir * light.area_radius,
                light.area_radius,
            ),
            RenderAreaLightShape::Tube => (
                right_dir * (light.area_width * 0.5),
                up_dir * light.area_radius,
                light.area_radius,
            ),
        };

        let shape = match light.area_shape {
            RenderAreaLightShape::Rectangle => AREA_SHAPE_RECTANGLE,
            RenderAreaLightShape::Disk => AREA_SHAPE_DISK,
            RenderAreaLightShape::Sphere => AREA_SHAPE_SPHERE,
            RenderAreaLightShape::Tube => AREA_SHAPE_TUBE,
        };

        entity_to_index.insert(entity, area_lights_data.len());
        area_lights_data.push(AreaLightData {
            position: [center.x, center.y, center.z, normal.x],
            right: [right_half.x, right_half.y, right_half.z, normal.y],
            up: [up_half.x, up_half.y, up_half.z, normal.z],
            color: [
                light.color.x * light.intensity,
                light.color.y * light.intensity,
                light.color.z * light.intensity,
                1.0,
            ],
            shape,
            range: light.range,
            radius,
            two_sided: u32::from(light.area_two_sided),
            shadow_index: -1,
            emissive_layer: AREA_EMISSIVE_LAYER_NONE,
            _pad0: 0.0,
            _pad1: 0.0,
        });
    }

    AreaLightCollectionResult {
        area_lights_data,
        entity_to_index,
    }
}

pub fn calculate_cascade_shadows(
    camera: Option<&CameraFrameInputs>,
    directional_light: Option<&(RenderLightData, nalgebra_glm::Mat4)>,
) -> CascadeShadowResult {
    let mut cascade_view_projections = [[[0.0f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES];
    let mut cascade_diameters = [0.0f32; crate::wgpu::passes::NUM_SHADOW_CASCADES];
    let mut cascade_split_distances = CASCADE_SPLIT_DISTANCES;

    let (light_view_projection, shadow_bias, shadow_normal_bias, light_size, shadows_enabled) =
        if let Some((light, light_transform)) = directional_light {
            let light_direction = transform_forward(light_transform);

            let camera_matrices = camera.and_then(|camera| camera.matrices);

            if let Some(camera_matrices) = camera_matrices {
                let (fov, aspect, camera_near) = camera
                    .map(|camera| {
                        (
                            camera.projection.y_fov_rad,
                            camera.projection.aspect,
                            camera.projection.z_near,
                        )
                    })
                    .unwrap_or((std::f32::consts::FRAC_PI_4, 1.78, 0.1));

                cascade_split_distances = scale_cascade_splits(camera_near);

                let cascade_resolution = crate::wgpu::passes::CASCADE_SLOT_RESOLUTION;

                for cascade_index in 0..crate::wgpu::passes::NUM_SHADOW_CASCADES {
                    let cascade_near = if cascade_index == 0 {
                        camera_near
                    } else {
                        cascade_split_distances[cascade_index - 1]
                    };
                    let cascade_far = cascade_split_distances[cascade_index];

                    let frustum_corners = get_frustum_corners_world_space(
                        &camera_matrices.view,
                        fov,
                        aspect,
                        cascade_near,
                        cascade_far,
                    );

                    let result = calculate_cascade_view_projection(
                        &frustum_corners,
                        &light_direction,
                        cascade_resolution,
                        cascade_far,
                    );

                    cascade_view_projections[cascade_index] = result.view_projection.into();
                    cascade_diameters[cascade_index] = result.cascade_diameter;
                }
            } else {
                let up = if light_direction.y.abs() > 0.99 {
                    nalgebra_glm::vec3(1.0, 0.0, 0.0)
                } else {
                    nalgebra_glm::vec3(0.0, 1.0, 0.0)
                };

                for cascade_index in 0..crate::wgpu::passes::NUM_SHADOW_CASCADES {
                    let cascade_far = cascade_split_distances[cascade_index];
                    let half_size = cascade_far * 0.5;

                    let scene_center = nalgebra_glm::vec3(0.0, 0.0, 0.0);
                    let light_position = scene_center - light_direction * 100.0;
                    let light_view = nalgebra_glm::look_at(&light_position, &scene_center, &up);
                    let light_projection = reverse_z_ortho_light(
                        -half_size,
                        half_size,
                        -half_size,
                        half_size,
                        0.1,
                        cascade_far * 2.0,
                    );

                    cascade_view_projections[cascade_index] =
                        (light_projection * light_view).into();
                    cascade_diameters[cascade_index] = cascade_far;
                }
            }

            let bias = if light.shadow_bias > 0.0 {
                light.shadow_bias
            } else {
                0.02
            };
            (
                cascade_view_projections[0],
                bias,
                light.shadow_normal_bias,
                light.shadow_softness,
                1.0,
            )
        } else {
            (
                nalgebra_glm::Mat4::identity().into(),
                0.0,
                default_shadow_normal_bias_value(),
                1.0,
                0.0,
            )
        };

    CascadeShadowResult {
        cascade_view_projections,
        cascade_diameters,
        cascade_split_distances,
        light_view_projection,
        shadow_bias,
        shadow_normal_bias,
        light_size,
        shadows_enabled,
    }
}

/// Collects the frame's full lighting: light lists, area lights, the cascade
/// shadow fit for the dispatching camera, and the sun.
pub fn build_render_lighting(
    scene: &RendererState,
    camera: Option<&CameraFrameInputs>,
) -> RenderLighting {
    const MAX_LIGHTS: usize = 1024;
    let light_result = collect_lights(scene, MAX_LIGHTS);
    let directional = light_result.directional_light;
    let directional_light_direction = directional
        .as_ref()
        .map(|(_light, transform)| {
            let dir = transform_forward(transform);
            [dir.x, dir.y, dir.z, 0.0]
        })
        .unwrap_or([0.0, -1.0, 0.0, 0.0]);
    let cascade = calculate_cascade_shadows(camera, directional.as_ref());
    let area = collect_area_lights(scene, MAX_AREA_LIGHTS);
    let mut sun = None;
    for render_light in &scene.render_lights.lights {
        let light = &render_light.light;
        let transform = &render_light.transform;
        if light.light_type == RenderLightType::Directional {
            sun = Some((
                -transform_forward(transform).normalize(),
                light.color * light.intensity,
            ));
            break;
        }
    }
    let (sun_direction, sun_color) = sun.unwrap_or((
        nalgebra_glm::vec3(0.35, 0.8, 0.35).normalize(),
        nalgebra_glm::vec3(4.0, 3.8, 3.2),
    ));
    RenderLighting {
        num_directional_lights: light_result.num_directional_lights,
        has_directional_light: directional.is_some(),
        entity_to_lights_index: light_result.entity_to_index,
        lights_data: light_result.lights_data,
        directional_light_direction,
        cascade_view_projections: cascade.cascade_view_projections,
        cascade_diameters: cascade.cascade_diameters,
        cascade_split_distances: cascade.cascade_split_distances,
        light_view_projection: cascade.light_view_projection,
        shadow_bias: cascade.shadow_bias,
        shadow_normal_bias: cascade.shadow_normal_bias,
        directional_light_size: cascade.light_size,
        shadows_enabled: cascade.shadows_enabled,
        area_lights_data: area.area_lights_data,
        area_entity_to_index: area.entity_to_index,
        sun_direction,
        sun_color,
    }
}
