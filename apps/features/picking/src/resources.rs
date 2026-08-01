use nightshade::prelude::*;
use nightshade_render::resources::gpu_picking::GpuPickResult;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickingMode {
    #[default]
    GpuHover,
    CpuHover,
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub panel: Entity,
    pub mode_button: Entity,
    pub debug_volumes_toggle: Entity,
    pub spin_slider: Entity,
    pub status_label: Entity,
    pub detail_label: Entity,
}

#[derive(Default)]
pub struct PickingResources {
    pub mode: PickingMode,
    pub highlighted_entity: Option<Entity>,
    pub selected_entity: Option<Entity>,
    pub last_pick_result: Option<GpuPickResult>,
    pub last_mouse_pos: (u32, u32),
    pub preview_lines_entity: Option<Entity>,
    pub show_debug_volumes: bool,
    pub spin_speed: f32,
    pub handles: UiHandles,
}

pub fn mode_label(mode: PickingMode) -> &'static str {
    match mode {
        PickingMode::GpuHover => "Mode: GPU hover",
        PickingMode::CpuHover => "Mode: CPU hover",
    }
}
