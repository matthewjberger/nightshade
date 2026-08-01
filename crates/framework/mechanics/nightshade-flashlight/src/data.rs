use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy)]
pub struct CellLow {
    pub entity: Entity,
}

#[derive(Debug, Clone, Copy)]
pub struct CellDied {
    pub entity: Entity,
}
