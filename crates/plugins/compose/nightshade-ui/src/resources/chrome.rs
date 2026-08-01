use super::*;
use nalgebra_glm;
use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct UiViewport {
    pub size: Vec2,
    pub breakpoint: UiBreakpoint,
}

#[derive(Default, Clone, Debug)]
pub struct WindowChrome {
    pub menu_bar: Option<nightshade::ecs::Entity>,
    pub toolbar: Option<nightshade::ecs::Entity>,
    pub status_bar: Option<nightshade::ecs::Entity>,
    pub shortcuts: Vec<WindowShortcut>,
}

#[derive(Clone, Debug)]
pub struct WindowShortcut {
    pub binding: crate::components::ShortcutBinding,
    pub command_index: usize,
}
