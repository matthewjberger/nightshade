use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventKind {
    Started,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub kind: CollisionEventKind,
    pub is_sensor: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ContactForceEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub total_force_magnitude: f32,
}
