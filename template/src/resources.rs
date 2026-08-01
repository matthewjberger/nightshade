#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: nightshade::ecs::Entity,
    pub rate_readout: nightshade::ecs::Entity,
    pub viewport_readout: nightshade::ecs::Entity,
    pub menu_button: nightshade::ecs::Entity,
    pub menu_panel: nightshade::ecs::Entity,
    pub extent_button: nightshade::ecs::Entity,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Default, Clone, Copy)]
pub struct ScreenHandles {
    pub play_button: nightshade::ecs::Entity,
    pub title_quit_button: nightshade::ecs::Entity,
    pub resume_button: nightshade::ecs::Entity,
    pub pause_title_button: nightshade::ecs::Entity,
}

#[derive(Default)]
pub struct TemplateResources {
    pub example: ExampleState,
    pub hud: HudHandles,
    pub screens: ScreenHandles,
    pub last_rate: u32,
    pub last_gpu_tenths: u32,
    pub last_viewport_rect: Option<nightshade_render::data::config::ViewportRect>,
    pub last_content_rect: Option<nightshade_platform::contract::window::WindowRect>,
    pub menu_open: bool,
    pub extent: nightshade_render::resources::viewport::ViewportExtent,
}

#[derive(Default)]
pub struct ExampleState {
    pub frame_count: u64,
    pub elapsed_seconds: f32,
    pub pong_count: u64,
}
