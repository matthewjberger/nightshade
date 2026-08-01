use nightshade::ecs::Entity;

#[derive(Default)]
pub struct ActionsResources {
    pub cube: Entity,
    pub rise: f32,
    pub lift: f32,
    pub listening: bool,
    pub shown_revision: Option<u64>,
    pub shown_held: String,
    pub shown_hint: String,
    pub profile_label: Entity,
    pub action_labels: Vec<Entity>,
    pub held_label: Entity,
    pub hint_label: Entity,
}
