use nightshade::prelude::*;
use std::collections::HashMap;

use crate::data::{EnemyType, GameState, TowerType, UiAction};
#[derive(Default)]
pub struct TowerDefenseResources {
    pub money: u32,
    pub lives: u32,
    pub wave: u32,
    pub game_state: GameState,
    pub selected_tower_type: TowerType,
    pub preview_entity: Option<Entity>,
    pub preview_range_lines: Vec<Entity>,
    pub ui_handles: UiHandles,
    pub ui_interaction: UiInteraction,
    pub spawn_timer: f32,
    pub enemies_to_spawn: Vec<EnemySpawnInfo>,
    pub wave_delay: f32,
    pub mouse_grid_pos: Option<(i32, i32)>,
    pub camera_entity: Entity,
    pub left_press: Option<Vec2>,
    pub right_press: Option<Vec2>,
    pub path: Vec<Vec3>,
    pub towers_by_position: HashMap<(i32, i32), Entity>,
    pub enemies_list: Vec<Entity>,
    pub projectiles_list: Vec<Entity>,
    pub effects_list: Vec<Entity>,
    pub hover_tower_text: Option<Entity>,
    pub grid_tiles: HashMap<(i32, i32), Entity>,
    pub tile_original_colors: HashMap<(i32, i32), Vec4>,
    pub last_hovered_tile: Option<(i32, i32)>,
    pub wave_announce_timer: f32,
    pub game_speed: f32,
    pub current_hp: u32,
    pub max_hp: u32,
    pub auto_start_waves: bool,
}

#[derive(Debug, Clone)]
pub struct EnemySpawnInfo {
    pub enemy_type: EnemyType,
    pub spawn_time: f32,
}

#[derive(Default, Clone)]
pub struct TowerChipHandles {
    pub root: Entity,
}

#[derive(Default)]
pub struct UiHandles {
    pub money_value: Entity,
    pub lives_value: Entity,
    pub wave_value: Entity,
    pub hp_value: Entity,
    pub hp_fill: Entity,
    pub speed_value: Entity,
    pub speed_minus: Entity,
    pub speed_reset: Entity,
    pub speed_plus: Entity,
    pub status_root: Entity,
    pub status_label: Entity,
    pub announce_root: Entity,
    pub announce_label: Entity,
    pub tower_chips: Vec<TowerChipHandles>,
}

#[derive(Default)]
pub struct UiInteraction {
    pub actions: Vec<UiAction>,
}
