use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy)]
pub struct PickedUp {
    pub collector: Entity,
    pub pickup: Entity,
    pub kind: u32,
    pub count: u32,
}
