use crate::data;
use nightshade::prelude::Entity;

pub struct PortalSettings {
    pub viewer: Entity,
    pub field_of_view: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub plane_bias: f32,
    pub target_max: u32,
    pub depth: u32,
    pub max_distance: f32,
}

impl Default for PortalSettings {
    fn default() -> Self {
        Self {
            viewer: Entity::default(),
            field_of_view: data::FIELD_OF_VIEW,
            near_plane: data::NEAR_PLANE,
            far_plane: data::FAR_PLANE,
            plane_bias: data::PLANE_BIAS,
            target_max: data::TARGET_MAX,
            depth: data::DEPTH,
            max_distance: data::MAX_DISTANCE,
        }
    }
}
