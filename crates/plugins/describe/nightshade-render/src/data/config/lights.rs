#[derive(Clone, Default)]
pub struct RenderLighting {
    pub lights_data: Vec<crate::data::light_gpu::LightData>,
    pub num_directional_lights: u32,
    pub directional_light_direction: [f32; 4],
    pub has_directional_light: bool,
    pub entity_to_lights_index: std::collections::HashMap<nightshade::ecs::Entity, usize>,
    pub cascade_view_projections: [[[f32; 4]; 4]; crate::data::light_gpu::NUM_SHADOW_CASCADES],
    pub cascade_diameters: [f32; crate::data::light_gpu::NUM_SHADOW_CASCADES],
    pub cascade_split_distances: [f32; crate::data::light_gpu::NUM_SHADOW_CASCADES],
    pub light_view_projection: [[f32; 4]; 4],
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub directional_light_size: f32,
    pub shadows_enabled: f32,
    pub area_lights_data: Vec<crate::data::light_gpu::AreaLightData>,
    pub area_entity_to_index: std::collections::HashMap<nightshade::ecs::Entity, usize>,
    pub sun_direction: nalgebra_glm::Vec3,
    pub sun_color: nalgebra_glm::Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderLightType {
    #[default]
    Directional,
    Point,
    Spot,
    Area,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderAreaLightShape {
    #[default]
    Rectangle,
    Disk,
    Sphere,
    Tube,
}

#[derive(Clone, Default)]
pub struct RenderShadowCasters {
    pub skinned: Vec<nightshade::ecs::Entity>,
}
