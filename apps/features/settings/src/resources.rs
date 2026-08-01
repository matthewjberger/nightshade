use nightshade::ecs::Entity;

#[derive(Default)]
pub struct SettingsResources {
    pub cubes: Vec<Entity>,
    pub angle: f32,
    pub title_label: Entity,
    pub shown_title: String,
    pub shown_palette: Option<usize>,
    pub shown_tint: Option<[f32; 4]>,
}
