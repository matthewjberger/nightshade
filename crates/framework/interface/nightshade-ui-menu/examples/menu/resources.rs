use nightshade::ecs::Entity;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
    Over,
}

#[derive(Default)]
pub struct Buttons {
    pub play: Entity,
    pub quit: Entity,
    pub resume: Entity,
    pub to_title: Entity,
    pub again: Entity,
}
