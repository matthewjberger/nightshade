use nalgebra_glm::Vec2;
use nightshade::ecs::Entity;

pub(crate) struct LineState {
    pub(crate) entity: Entity,
    pub(crate) text_slot: usize,
    pub(crate) text: String,
    pub(crate) is_command: bool,
    pub(crate) visible: bool,
}

#[derive(Default)]
pub struct ShellOverlayState(pub Option<ShellOverlay>);

pub struct ShellOverlay {
    pub(crate) panel: Entity,
    pub(crate) scroll: Entity,
    pub(crate) content: Entity,
    pub(crate) lines: Vec<LineState>,
    pub(crate) input_slot: usize,
    pub(crate) resize_handle: Entity,
    pub(crate) panel_visible: bool,
    pub(crate) panel_size: Vec2,
    pub(crate) input_cache: String,
    pub(crate) handle_active: bool,
    pub(crate) last_content_height: f32,
    pub(crate) last_output_len: usize,
    pub(crate) forced_ui_visible: bool,
}
