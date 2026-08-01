use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Swing {
    pub swinger: Option<Entity>,
    pub targets: Vec<Entity>,
    pub yaw: f32,
    pub half_angle: f32,
    pub reach: f32,
}
