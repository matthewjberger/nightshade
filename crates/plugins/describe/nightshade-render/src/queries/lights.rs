use crate::components::light::RenderLightData;
use crate::data::config::{
    CameraFrameInputs, RenderAreaLightShape, RenderLightType, RenderLighting, RenderView,
};
use crate::data::light_gpu::{AreaLightData, COOKIE_LAYER_NONE, LightData, NUM_SHADOW_CASCADES};
use crate::data::shadow::CASCADE_SPLIT_DISTANCES;
use crate::queries::projection::{
    cascade_view_projection_without_a_camera, fit_cascades, transform_forward, transform_right,
    transform_translation, transform_up,
};

pub const AREA_SHAPE_TUBE: u32 = 3;

pub const AREA_SHAPE_SPHERE: u32 = 2;

pub const AREA_SHAPE_DISK: u32 = 1;

pub const AREA_SHAPE_RECTANGLE: u32 = 0;

pub const AREA_EMISSIVE_LAYER_NONE: u32 = 0xFFFFFFFF;

pub struct LightCollectionResult {
    pub lights_data: Vec<LightData>,
    pub directional_light: Option<(
        crate::components::light::RenderLightData,
        nalgebra_glm::Mat4,
    )>,
    pub num_directional_lights: u32,
    pub entity_to_index: std::collections::HashMap<nightshade::ecs::Entity, usize>,
}

pub struct AreaLightCollectionResult {
    pub area_lights_data: Vec<AreaLightData>,
    pub entity_to_index: std::collections::HashMap<nightshade::ecs::Entity, usize>,
}

pub struct CascadeShadowResult {
    pub cascade_view_projections: [[[f32; 4]; 4]; NUM_SHADOW_CASCADES],
    pub cascade_diameters: [f32; NUM_SHADOW_CASCADES],
    pub cascade_split_distances: [f32; NUM_SHADOW_CASCADES],
    pub light_view_projection: [[f32; 4]; 4],
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub light_size: f32,
    pub shadows_enabled: f32,
}

pub fn default_shadow_normal_bias_value() -> f32 {
    1.8
}

pub fn collect_lights(
    world: &nightshade::ecs::Storage,
    max_lights: usize,
    eye: Option<nalgebra_glm::Vec3>,
) -> LightCollectionResult {
    let mut directional_lights = Vec::new();
    let mut local_lights = Vec::new();
    let mut directional_light = None;
    let mut directional_entities = Vec::new();
    let mut local_entities = Vec::new();
    let mut local_positions: Vec<nalgebra_glm::Vec3> = Vec::new();

    let sources: Vec<(nightshade::ecs::Entity, RenderLightData, nalgebra_glm::Mat4)> = world
        .query_ref::<(
            &RenderLightData,
            &crate::components::render_world::Transform,
        )>()
        .iter()
        .map(|(entity, (light, transform))| (entity, light.clone(), transform.0))
        .collect();
    for (entity, light, transform) in &sources {
        let entity = *entity;
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
                casts_shadow: if light.cast_shadows { 1.0 } else { 0.0 },
            };

            if light_type == 0 {
                directional_lights.push(light_data);
                directional_entities.push(entity);
            } else {
                local_lights.push(light_data);
                local_entities.push(entity);
                local_positions.push(position);
            }
        }
    }

    let room = max_lights.saturating_sub(directional_lights.len());
    if local_lights.len() > room {
        let mut order: Vec<usize> = (0..local_lights.len()).collect();
        if let Some(eye) = eye {
            order.sort_by(|left, right| {
                let near = (local_positions[*left] - eye).magnitude_squared();
                let far = (local_positions[*right] - eye).magnitude_squared();
                near.total_cmp(&far)
            });
        }
        order.truncate(room);
        local_lights = order.iter().map(|&index| local_lights[index]).collect();
        local_entities = order.iter().map(|&index| local_entities[index]).collect();
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
    world: &nightshade::ecs::Storage,
    max_area_lights: usize,
) -> AreaLightCollectionResult {
    let mut area_lights_data = Vec::new();
    let mut entity_to_index = std::collections::HashMap::new();

    let sources: Vec<(nightshade::ecs::Entity, RenderLightData, nalgebra_glm::Mat4)> = world
        .query_ref::<(
            &RenderLightData,
            &crate::components::render_world::Transform,
        )>()
        .iter()
        .map(|(entity, (light, transform))| (entity, light.clone(), transform.0))
        .collect();
    for (entity, light, transform) in &sources {
        if area_lights_data.len() >= max_area_lights {
            break;
        }

        let entity = *entity;
        let transform = &transform;

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
    cascade_resolution: f32,
) -> CascadeShadowResult {
    let mut cascade_view_projections = [[[0.0f32; 4]; 4]; NUM_SHADOW_CASCADES];
    let mut cascade_diameters = [0.0f32; NUM_SHADOW_CASCADES];
    let mut cascade_split_distances = CASCADE_SPLIT_DISTANCES;

    let (light_view_projection, shadow_bias, shadow_normal_bias, light_size, shadows_enabled) =
        if let Some((light, light_transform)) = directional_light {
            let light_direction = transform_forward(light_transform);

            let camera_matrices = camera.and_then(|camera| camera.matrices);

            if let Some(camera_matrices) = camera_matrices {
                let projection = camera.map(|camera| camera.projection).unwrap_or_default();
                let fit = fit_cascades(
                    &camera_matrices.view,
                    &projection,
                    &light_direction,
                    cascade_resolution,
                );
                cascade_view_projections = fit.view_projections;
                cascade_diameters = fit.diameters;
                cascade_split_distances = fit.split_distances;
            } else {
                for cascade_index in 0..NUM_SHADOW_CASCADES {
                    let cascade_far = cascade_split_distances[cascade_index];
                    cascade_view_projections[cascade_index] =
                        cascade_view_projection_without_a_camera(&light_direction, cascade_far)
                            .into();
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

pub fn build_render_lighting(
    world: &nightshade::ecs::Storage,
    camera: Option<&CameraFrameInputs>,
    max_lights: usize,
    max_area_lights: usize,
    cascade_resolution: f32,
) -> RenderLighting {
    let eye = camera
        .and_then(|camera| camera.world_transform.as_ref())
        .map(transform_translation);
    let light_result = collect_lights(world, max_lights, eye);
    let directional = light_result.directional_light;
    let directional_light_direction = directional
        .as_ref()
        .map(|(_light, transform)| {
            let dir = transform_forward(transform);
            [dir.x, dir.y, dir.z, 0.0]
        })
        .unwrap_or([0.0, -1.0, 0.0, 0.0]);
    let cascade = calculate_cascade_shadows(camera, directional.as_ref(), cascade_resolution);
    let area = collect_area_lights(world, max_area_lights);
    let mut sun = None;
    for (_entity, (light, transform)) in world
        .query_ref::<(
            &RenderLightData,
            &crate::components::render_world::Transform,
        )>()
        .iter()
    {
        if light.light_type == RenderLightType::Directional {
            sun = Some((
                -transform_forward(&transform.0).normalize(),
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

pub fn sun_rotation(turn: f32, tilt: f32) -> nalgebra_glm::Quat {
    nalgebra_glm::quat_angle_axis(turn, &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0))
        * nalgebra_glm::quat_angle_axis(tilt, &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0))
}

pub fn cascade_texel_sizes(lighting: &RenderLighting, slot_size: u32) -> [f32; 4] {
    let mut sizes = [0.0f32; 4];
    for (cascade, diameter) in lighting.cascade_diameters.iter().enumerate() {
        sizes[cascade] = diameter / slot_size as f32;
    }
    sizes
}

pub fn shadows_enabled(lighting: &RenderLighting) -> f32 {
    if lighting.shadows_enabled > 0.0 && lighting.has_directional_light {
        1.0
    } else {
        0.0
    }
}

pub fn screen_coverage(light: &LightData, reach: f32, view: &RenderView) -> f32 {
    let position = nalgebra_glm::vec3(light.position[0], light.position[1], light.position[2]);
    let distance = nalgebra_glm::distance(&position, &view.camera_position);
    if distance <= reach {
        return 1.0;
    }
    let focal = view.projection[(1, 1)];
    let height = view.screen_size.1.max(1) as f32;
    let tangent = reach / (distance * distance - reach * reach).sqrt();
    let radius_pixels = tangent * focal * height * 0.5;
    let width = view.screen_size.0.max(1) as f32;
    (std::f32::consts::PI * radius_pixels * radius_pixels / (width * height)).clamp(0.0, 1.0)
}

pub fn wanted_slot(coverage: f32, view: &RenderView, widest: u32, narrowest: u32) -> u32 {
    let rows = coverage.sqrt() * view.screen_size.1.max(1) as f32;
    let mut size = widest;
    while size > narrowest && (size as f32) > rows {
        size /= 2;
    }
    size
}
