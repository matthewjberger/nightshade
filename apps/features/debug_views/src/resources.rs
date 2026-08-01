use nightshade::prelude::*;
use nightshade_render::data::config::PbrDebugMode;

#[derive(Default, Clone)]
pub struct UiHandles {
    pub panel: Entity,
    pub mode_label: Entity,
    pub buttons: Vec<(PbrDebugMode, Entity)>,
}

#[derive(Default)]
pub struct DebugViewsResources {
    pub handles: UiHandles,
    pub model: Option<nightshade_load::LoadHandle>,
}
