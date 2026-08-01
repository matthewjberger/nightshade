use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy)]
pub struct Hurt {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Died {
    pub entity: Entity,
    pub bounty: u32,
}
