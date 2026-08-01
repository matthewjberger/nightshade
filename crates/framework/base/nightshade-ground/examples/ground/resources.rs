use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Probes {
    pub probes: Vec<Entity>,
    pub walker: Option<Entity>,
    pub inside: bool,
}
