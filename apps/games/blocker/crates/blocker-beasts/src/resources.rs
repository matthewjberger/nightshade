use nightshade::ecs::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Quarry(pub Option<Entity>);
