//! CPU-side collection of the frame's light lists from [`RendererState`]
//! into the packed GPU structures the mesh and shadow passes consume.

use crate::config::{RenderAreaLightShape, RenderLightType, RendererState};
use crate::wgpu::passes::geometry::projection::{
    AREA_EMISSIVE_LAYER_NONE, AREA_SHAPE_DISK, AREA_SHAPE_RECTANGLE, AREA_SHAPE_SPHERE,
    AREA_SHAPE_TUBE, AreaLightCollectionResult, AreaLightData, COOKIE_LAYER_NONE,
    LightCollectionResult, LightData, transform_forward, transform_right, transform_translation,
    transform_up,
};

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
