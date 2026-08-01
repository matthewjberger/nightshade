use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct RenderSceneSync {
    pub initialized: bool,
    pub structural_cursor: u64,
    pub data_cursor: u32,
    pub materials_dirty: bool,
    pub materials_initialized: bool,
    pub materials_version_seen: u64,
    pub materials_names_version_seen: u64,
    pub materials_missing_named: HashSet<String>,
    pub animation_initialized: bool,
    pub animation_cursor: u32,
    pub animation_skinned_generation_seen: u64,
    pub animation_player_clips: HashMap<nightshade::ecs::Entity, (i64, i64)>,
    pub animation_player_clips_scratch: HashMap<nightshade::ecs::Entity, (i64, i64)>,
}
