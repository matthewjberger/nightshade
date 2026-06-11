use nightshade::prelude::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Title,
    InGame,
    Paused,
}

#[derive(Default)]
pub struct ScreenState {
    pub current: Screen,
}

#[derive(Default)]
pub struct PlayerState {
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
}

#[derive(Default)]
pub struct TitleHandles {
    pub root: Entity,
    pub play_button: Entity,
    pub quit_button: Entity,
}

#[derive(Default)]
pub struct PauseHandles {
    pub root: Entity,
    pub resume_button: Entity,
    pub main_menu_button: Entity,
    pub quit_button: Entity,
    pub infinite_world_checkbox: Entity,
    pub grass_checkbox: Entity,
}

#[derive(Default)]
pub struct InfiniteWorldState {
    pub enabled: bool,
    pub ground: Option<Entity>,
}

#[derive(Default)]
pub struct HudHandles {
    pub root: Entity,
    pub fps_label: Entity,
    pub crosshair: Entity,
}

#[derive(Default)]
pub struct UiHandles {
    pub title: TitleHandles,
    pub pause: PauseHandles,
    pub hud: HudHandles,
}
