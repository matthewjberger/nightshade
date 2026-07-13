use nightshade::prelude::*;
use std::collections::HashMap;

freecs::dynamic_schema! {
    pub fn register_tower_defense_components {
        position: Position => POSITION,
        tower: Tower => TOWER,
        enemy: Enemy => ENEMY,
        projectile: Projectile => PROJECTILE,
        grid_cell: GridCell => GRID_CELL,
        visual_effect: VisualEffect => VISUAL_EFFECT,
        range_indicator: RangeIndicator => RANGE_INDICATOR,
        money_popup: MoneyPopup => MONEY_POPUP,
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TowerType {
    #[default]
    Basic,
    Frost,
    Cannon,
    Sniper,
    Poison,
}

impl TowerType {
    pub fn cost(&self) -> u32 {
        match self {
            TowerType::Basic => 60,
            TowerType::Frost => 120,
            TowerType::Cannon => 200,
            TowerType::Sniper => 180,
            TowerType::Poison => 150,
        }
    }

    pub fn damage(&self) -> f32 {
        match self {
            TowerType::Basic => 15.0,
            TowerType::Frost => 8.0,
            TowerType::Cannon => 50.0,
            TowerType::Sniper => 80.0,
            TowerType::Poison => 5.0,
        }
    }

    pub fn range(&self) -> f32 {
        match self {
            TowerType::Basic => 3.0,
            TowerType::Frost => 2.5,
            TowerType::Cannon => 4.0,
            TowerType::Sniper => 6.0,
            TowerType::Poison => 2.8,
        }
    }

    pub fn fire_rate(&self) -> f32 {
        match self {
            TowerType::Basic => 0.5,
            TowerType::Frost => 1.0,
            TowerType::Cannon => 2.0,
            TowerType::Sniper => 3.0,
            TowerType::Poison => 0.8,
        }
    }

    pub fn color(&self) -> Vec4 {
        match self {
            TowerType::Basic => nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            TowerType::Frost => nalgebra_glm::vec4(0.2, 0.6, 1.0, 1.0),
            TowerType::Cannon => nalgebra_glm::vec4(0.8, 0.2, 0.2, 1.0),
            TowerType::Sniper => nalgebra_glm::vec4(0.3, 0.3, 0.3, 1.0),
            TowerType::Poison => nalgebra_glm::vec4(0.6, 0.2, 0.8, 1.0),
        }
    }

    pub fn projectile_speed(&self) -> f32 {
        match self {
            TowerType::Basic => 12.0,
            TowerType::Frost => 8.0,
            TowerType::Cannon => 10.0,
            TowerType::Sniper => 20.0,
            TowerType::Poison => 10.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TowerType::Basic => "Basic",
            TowerType::Frost => "Frost",
            TowerType::Cannon => "Cannon",
            TowerType::Sniper => "Sniper",
            TowerType::Poison => "Poison",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TowerType::Basic => "Balanced fire",
            TowerType::Frost => "Slows on hit",
            TowerType::Cannon => "Splash damage",
            TowerType::Sniper => "Charged shot",
            TowerType::Poison => "Damage over time",
        }
    }

    pub fn hotkey_label(&self) -> &'static str {
        match self {
            TowerType::Basic => "1",
            TowerType::Frost => "2",
            TowerType::Cannon => "3",
            TowerType::Sniper => "4",
            TowerType::Poison => "5",
        }
    }

    pub fn all() -> [TowerType; 5] {
        [
            TowerType::Basic,
            TowerType::Frost,
            TowerType::Cannon,
            TowerType::Sniper,
            TowerType::Poison,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    WaitingForWave,
    WaveInProgress,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EnemyType {
    #[default]
    Normal,
    Fast,
    Tank,
    Flying,
    Shielded,
    Healer,
    Boss,
}

impl EnemyType {
    pub fn base_health(&self) -> f32 {
        match self {
            EnemyType::Normal => 50.0,
            EnemyType::Fast => 30.0,
            EnemyType::Tank => 150.0,
            EnemyType::Flying => 40.0,
            EnemyType::Shielded => 60.0,
            EnemyType::Healer => 80.0,
            EnemyType::Boss => 500.0,
        }
    }

    pub fn health(&self, wave: u32) -> f32 {
        let health_multiplier = 1.0 + (wave as f32 - 1.0) * 0.5;
        self.base_health() * health_multiplier
    }

    pub fn speed(&self) -> f32 {
        match self {
            EnemyType::Normal => 2.0,
            EnemyType::Fast => 4.0,
            EnemyType::Tank => 1.0,
            EnemyType::Flying => 2.5,
            EnemyType::Shielded => 1.5,
            EnemyType::Healer => 1.8,
            EnemyType::Boss => 0.8,
        }
    }

    pub fn value(&self, wave: u32) -> u32 {
        let base = match self {
            EnemyType::Normal => 10,
            EnemyType::Fast => 15,
            EnemyType::Tank => 30,
            EnemyType::Flying => 20,
            EnemyType::Shielded => 25,
            EnemyType::Healer => 35,
            EnemyType::Boss => 100,
        };
        base + wave * 2
    }

    pub fn color(&self) -> Vec4 {
        match self {
            EnemyType::Normal => nalgebra_glm::vec4(0.8, 0.2, 0.2, 1.0),
            EnemyType::Fast => nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            EnemyType::Tank => nalgebra_glm::vec4(0.4, 0.4, 0.4, 1.0),
            EnemyType::Flying => nalgebra_glm::vec4(0.5, 0.8, 1.0, 1.0),
            EnemyType::Shielded => nalgebra_glm::vec4(0.2, 0.6, 0.9, 1.0),
            EnemyType::Healer => nalgebra_glm::vec4(0.2, 0.9, 0.4, 1.0),
            EnemyType::Boss => nalgebra_glm::vec4(0.6, 0.0, 0.6, 1.0),
        }
    }

    pub fn scale(&self) -> Vec3 {
        match self {
            EnemyType::Normal => nalgebra_glm::vec3(0.4, 0.6, 0.4),
            EnemyType::Fast => nalgebra_glm::vec3(0.3, 0.5, 0.3),
            EnemyType::Tank => nalgebra_glm::vec3(0.6, 0.6, 0.6),
            EnemyType::Flying => nalgebra_glm::vec3(0.5, 0.5, 0.5),
            EnemyType::Shielded => nalgebra_glm::vec3(0.5, 0.5, 0.5),
            EnemyType::Healer => nalgebra_glm::vec3(0.4, 0.4, 0.4),
            EnemyType::Boss => nalgebra_glm::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn y_offset(&self) -> f32 {
        let base_offset = self.scale().y / 2.0 - 0.5;
        match self {
            EnemyType::Flying => 2.0 + base_offset,
            _ => base_offset,
        }
    }

    pub fn shield(&self) -> f32 {
        match self {
            EnemyType::Shielded => 30.0,
            EnemyType::Boss => 100.0,
            _ => 0.0,
        }
    }
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

#[derive(Debug, Clone)]
pub enum UiAction {
    SelectTower(TowerType),
    DecreaseSpeed,
    IncreaseSpeed,
    ResetSpeed,
}

#[derive(Default)]
pub struct UiInteraction {
    pub actions: Vec<UiAction>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Tower {
    pub tower_type: TowerType,
    pub cooldown: f32,
    pub target: Option<Entity>,
    pub fire_animation: f32,
    pub tracking_time: f32,
    pub target_line: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GridCell {
    pub x: i32,
    pub z: i32,
    pub occupied: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RangeIndicator {
    pub line_entities: Vec<Entity>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MoneyPopup {
    pub lifetime: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Enemy {
    pub health: f32,
    pub shield_health: f32,
    pub speed: f32,
    pub path_index: usize,
    pub path_progress: f32,
    pub value: u32,
    pub enemy_type: EnemyType,
    pub slow_duration: f32,
    pub poison_duration: f32,
    pub poison_damage: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Projectile {
    pub damage: f32,
    pub target: Entity,
    pub speed: f32,
    pub tower_type: TowerType,
    pub start_position: Vec3,
    pub arc_height: f32,
    pub flight_progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EffectType {
    #[default]
    Explosion,
    PoisonBubble,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VisualEffect {
    pub effect_type: EffectType,
    pub lifetime: f32,
    pub age: f32,
}
