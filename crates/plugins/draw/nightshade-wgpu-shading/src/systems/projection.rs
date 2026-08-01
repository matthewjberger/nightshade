pub use nightshade_render::queries::projection::*;

pub use nightshade_render::data::light_gpu::LightData;

pub use nightshade_render::data::shadow::CASCADE_SPLIT_DISTANCES;

pub use nightshade_wgpu::backend::caps::MAX_SPOTLIGHT_SHADOWS;

pub use nightshade_wgpu::backend::caps::{MAX_AREA_LIGHTS, MAX_LIGHTS};

pub use nightshade_render::data::light_gpu::AreaLightData;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AreaUniforms {
    pub count: u32,
    pub padding_0: u32,
    pub padding_1: u32,
    pub padding_2: u32,
}

pub fn resolve_cookie_layers(
    configs: &nightshade_wgpu::backend::RenderInputs,
    lights_data: &mut [LightData],
    entity_to_index: &std::collections::HashMap<nightshade::ecs::Entity, usize>,
    material_layer_map: &std::collections::HashMap<
        nightshade_render::data::asset_id::TextureId,
        nightshade_wgpu::backend::MaterialTextureLayer,
    >,
    texture_registry: &nightshade_render::data::generational_registry::GenerationalRegistry<()>,
) {
    for (entity, &index) in entity_to_index {
        let Some(light) = configs
            .scene_world
            .get::<nightshade_render::components::light::RenderLightData>(*entity)
        else {
            continue;
        };
        let Some(cookie_name) = light.cookie_texture.as_deref() else {
            continue;
        };
        if cookie_name.is_empty() {
            continue;
        }
        let Some((texture_index, _)) =
            nightshade_render::data::generational_registry::registry_lookup_index(
                texture_registry,
                cookie_name,
            )
        else {
            continue;
        };
        let texture_id = nightshade_render::data::asset_id::TextureId::new(texture_index, 0);
        if let Some(entry) = material_layer_map.get(&texture_id)
            && let Some(slot) = lights_data.get_mut(index)
        {
            slot.cookie_layer = entry.packed();
        }
    }
}

pub fn resolve_area_emissive_layers(
    configs: &nightshade_wgpu::backend::RenderInputs,
    area_lights_data: &mut [AreaLightData],
    entity_to_index: &std::collections::HashMap<nightshade::ecs::Entity, usize>,
    material_layer_map: &std::collections::HashMap<
        nightshade_render::data::asset_id::TextureId,
        nightshade_wgpu::backend::MaterialTextureLayer,
    >,
    texture_registry: &nightshade_render::data::generational_registry::GenerationalRegistry<()>,
) {
    for (entity, &index) in entity_to_index {
        let Some(light) = configs
            .scene_world
            .get::<nightshade_render::components::light::RenderLightData>(*entity)
        else {
            continue;
        };
        let Some(texture_name) = light.area_emissive_texture.as_deref() else {
            continue;
        };
        if texture_name.is_empty() {
            continue;
        }
        let Some((texture_index, _)) =
            nightshade_render::data::generational_registry::registry_lookup_index(
                texture_registry,
                texture_name,
            )
        else {
            continue;
        };
        let texture_id = nightshade_render::data::asset_id::TextureId::new(texture_index, 0);
        if let Some(entry) = material_layer_map.get(&texture_id)
            && let Some(slot) = area_lights_data.get_mut(index)
        {
            slot.emissive_layer = entry.packed();
        }
    }
}

pub struct SpotlightShadowResult {
    pub shadow_data: Vec<nightshade_render::data::shadow::SpotlightShadowData>,
    pub entity_to_shadow_index: std::collections::HashMap<nightshade::ecs::Entity, i32>,
}

pub use nightshade_render::queries::shadow::light_casts_atlas_shadow;

pub use nightshade_render::queries::shadow::atlas_shadow_fov;

pub fn collect_spotlight_shadows(
    configs: &nightshade_wgpu::backend::RenderInputs,
) -> SpotlightShadowResult {
    let assignment = &configs.owned.shadow_atlas;
    SpotlightShadowResult {
        shadow_data: assignment.data.clone(),
        entity_to_shadow_index: assignment.entity_to_index.clone(),
    }
}

pub fn apply_spotlight_shadow_indices(
    lights_data: &mut [LightData],
    entity_to_shadow_index: &std::collections::HashMap<nightshade::ecs::Entity, i32>,
    entity_to_lights_index: &std::collections::HashMap<nightshade::ecs::Entity, usize>,
) {
    for (entity, &shadow_index) in entity_to_shadow_index {
        if let Some(&lights_index) = entity_to_lights_index.get(entity)
            && lights_index < lights_data.len()
        {
            lights_data[lights_index].shadow_index = shadow_index;
        }
    }
}

pub fn collect_point_light_shadows(
    configs: &nightshade_wgpu::backend::RenderInputs,
    lights_data: &mut [LightData],
    entity_to_lights_index: &std::collections::HashMap<nightshade::ecs::Entity, usize>,
) -> Vec<crate::systems::light::PointLightShadowData> {
    let mut shadow_data: Vec<crate::systems::light::PointLightShadowData> = Vec::new();

    for chosen in &configs.owned.point_shadows {
        let transform = &chosen.transform;
        let light_position =
            nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);

        shadow_data.push(crate::systems::light::PointLightShadowData {
            position: [light_position.x, light_position.y, light_position.z],
            range: chosen.light.range.max(0.1),
            bias: chosen.light.shadow_bias,
            shadow_index: chosen.slot_index as i32,
            padding: [0.0; 2],
        });

        if let Some(&lights_index) = entity_to_lights_index.get(&chosen.entity)
            && lights_index < lights_data.len()
        {
            lights_data[lights_index].shadow_index = chosen.slot_index as i32;
        }
    }

    shadow_data
}

pub fn cascade_atlas_offsets(
    cascade_diameters: &[f32; nightshade_render::data::light_gpu::NUM_SHADOW_CASCADES],
) -> [[f32; 4]; nightshade_render::data::light_gpu::NUM_SHADOW_CASCADES] {
    let resolution = nightshade_wgpu::backend::caps::CASCADE_SLOT_RESOLUTION;
    [
        [0.0, 0.0, cascade_diameters[0] / resolution, 0.0],
        [0.5, 0.0, cascade_diameters[1] / resolution, 0.0],
        [0.0, 0.5, cascade_diameters[2] / resolution, 0.0],
        [0.5, 0.5, cascade_diameters[3] / resolution, 0.0],
    ]
}
