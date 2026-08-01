use crate::data::light_gpu::AreaLightData;

pub fn apply_area_light_shadow_indices(
    area_lights_data: &mut [AreaLightData],
    entity_to_shadow_index: &std::collections::HashMap<nightshade::ecs::Entity, i32>,
    entity_to_area_index: &std::collections::HashMap<nightshade::ecs::Entity, usize>,
) {
    for (entity, &shadow_index) in entity_to_shadow_index {
        if let Some(&area_index) = entity_to_area_index.get(entity)
            && area_index < area_lights_data.len()
        {
            area_lights_data[area_index].shadow_index = shadow_index;
        }
    }
}
