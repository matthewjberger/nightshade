use nightshade::prelude::Entity;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct RockObb {
    pub center: nalgebra_glm::Vec3,
    pub yaw: f32,
    pub half_extents: [f32; 3],
}

pub struct PropLod {
    pub center: nalgebra_glm::Vec3,
    pub detail_meshes: Vec<Entity>,
    pub mid_meshes: Vec<Entity>,
    pub cube: Entity,
    pub zone: u8,
}

#[derive(Default)]
pub struct RocksState {
    pub enabled: bool,
    pub lod_enabled: bool,
    pub models: Vec<nightshade::ecs::prefab::GltfLoadResult>,
    pub mid_models: Vec<nightshade::ecs::prefab::GltfLoadResult>,
    pub tiles: HashMap<(i32, i32), Vec<Entity>>,
    pub obbs: HashMap<(i32, i32), Vec<RockObb>>,
    pub lods: HashMap<(i32, i32), Vec<PropLod>>,
}

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
    pub show_fps: bool,
}

#[derive(Default)]
pub struct PlayerState {
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub fly_camera: Option<Entity>,
    pub fly: bool,
}

#[derive(Default)]
pub struct CharacterState {
    pub third_person: bool,
    pub body: Option<Entity>,
    pub camera: Option<Entity>,
    pub spheres: Option<Entity>,
    pub velocity: nalgebra_glm::Vec3,
    pub grounded: bool,
    pub jump_suppress_timer: f32,
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
    pub terrain_checkbox: Entity,
    pub third_person_checkbox: Entity,
    pub fly_checkbox: Entity,
    pub rocks_checkbox: Entity,
    pub prop_lod_checkbox: Entity,
    pub show_fps_checkbox: Entity,
    pub main_panel: Entity,
    pub settings_panel: Entity,
    pub settings_button: Entity,
    pub back_button: Entity,
}

#[derive(Default)]
pub struct InfiniteWorldState {
    pub enabled: bool,
    pub suppressed: bool,
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
