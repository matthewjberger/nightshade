use std::collections::HashMap;

#[derive(Default)]
pub struct RetainedUiAccessibility {
    pub reduced_motion: bool,
    pub announce_queue: Vec<String>,
    pub test_id_map: HashMap<String, nightshade::ecs::Entity>,
    pub named_entities: HashMap<String, nightshade::ecs::Entity>,
}

#[derive(Default)]
pub struct RetainedUiGroups {
    pub radio: HashMap<u32, Vec<nightshade::ecs::Entity>>,
    pub selectable_labels: HashMap<u32, Vec<nightshade::ecs::Entity>>,
    pub shortcuts: Vec<(crate::components::ShortcutBinding, usize)>,
}
