use nightshade::prelude::Entity;

#[derive(Default)]
pub struct TetherResources {
    pub run: RunState,
    pub view: ViewState,
    pub scene: SceneHandles,
    pub ui: UiHandles,
    pub screens: ScreenHandles,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Default)]
pub struct RunState {
    pub best_speed: f32,
    pub last_release: f32,
    pub falls: u32,
}

pub struct ViewState {
    pub shown_speed: i32,
    pub shown_kept: i32,
    pub shown_best: i32,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            shown_speed: -1,
            shown_kept: -1,
            shown_best: -1,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct SceneHandles {
    pub runner: Option<Entity>,
    pub camera: Option<Entity>,
    pub rope: Option<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub root: Option<Entity>,
    pub speed_readout: Option<Entity>,
    pub best_readout: Option<Entity>,
    pub meter_fill: Option<Entity>,
    pub reticle: Option<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct ScreenHandles {
    pub play_button: Entity,
    pub title_quit_button: Entity,
    pub resume_button: Entity,
    pub restart_button: Entity,
    pub pause_title_button: Entity,
    pub pause_quit_button: Entity,
}
