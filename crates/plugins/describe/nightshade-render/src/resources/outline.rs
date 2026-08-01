use nightshade::ecs::Entity;

#[derive(Clone)]
pub struct Outline {
    pub color: [f32; 4],
    pub active_entity: Option<Entity>,
    pub entities: Vec<Entity>,
}

pub const DEFAULT_OUTLINE_COLOR: [f32; 4] = [1.0, 0.45, 0.0, 1.0];

impl Default for Outline {
    fn default() -> Self {
        Self {
            color: DEFAULT_OUTLINE_COLOR,
            active_entity: None,
            entities: Vec::new(),
        }
    }
}
