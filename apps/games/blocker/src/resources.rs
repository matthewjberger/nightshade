use nightshade::ecs::Entity;

#[derive(Default)]
pub struct Showroom {
    pub body: Option<Entity>,
    pub wall: Option<Entity>,
    pub still: f32,
    pub gazing: bool,
}
