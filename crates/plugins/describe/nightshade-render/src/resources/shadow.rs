use crate::data::shadow::{SpotlightAtlasSlot, SpotlightShadowData};
use nightshade::ecs::Entity;
use std::collections::HashMap;

pub struct PointShadowSlot {
    pub entity: Entity,
    pub light: crate::components::light::RenderLightData,
    pub transform: nalgebra_glm::Mat4,
    pub slot_index: u32,
}

#[derive(Default)]
pub struct SpotlightAtlasAssignment {
    pub slots: Vec<SpotlightAtlasSlot>,
    pub data: Vec<SpotlightShadowData>,
    pub entity_to_index: HashMap<Entity, i32>,
    pub default_slot_size: u32,
}
