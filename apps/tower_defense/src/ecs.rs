use nightshade::prelude::*;
use std::collections::HashMap;

pub use freecs::Entity;

freecs::ecs! {
    GameWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        position: Position => POSITION,
        tower: Tower => TOWER,
        enemy: Enemy => ENEMY,
        projectile: Projectile => PROJECTILE,
        grid_cell: GridCell => GRID_CELL,
        visual_effect: VisualEffect => VISUAL_EFFECT,
        range_indicator: RangeIndicator => RANGE_INDICATOR,
        money_popup: MoneyPopup => MONEY_POPUP,
    }
    GameResources {
        money: u32,
        lives: u32,
        wave: u32,
        game_state: GameState,
        selected_tower_type: TowerType,
        preview_entity: Option<Entity>,
        preview_range_lines: Vec<Entity>,
        ui_handles: UiHandles,
        spawn_timer: f32,
        enemies_to_spawn: Vec<EnemySpawnInfo>,
        wave_delay: f32,
        mouse_grid_pos: Option<(i32, i32)>,
        camera_entity: Entity,
        path: Vec<Vec3>,
        towers_by_position: HashMap<(i32, i32), freecs::Entity>,
        enemies_list: Vec<freecs::Entity>,
        projectiles_list: Vec<freecs::Entity>,
        effects_list: Vec<freecs::Entity>,
        hover_tower_text: Option<Entity>,
        grid_tiles: HashMap<(i32, i32), Entity>,
        tile_original_colors: HashMap<(i32, i32), Vec4>,
        last_hovered_tile: Option<(i32, i32)>,
        wave_announce_timer: f32,
        game_speed: f32,
        current_hp: u32,
        max_hp: u32,
        auto_start_waves: bool,
    }
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

#[derive(Default)]
pub struct UiHandles {
    pub money_text: Option<Entity>,
    pub lives_text: Option<Entity>,
    pub wave_text: Option<Entity>,
    pub hp_text: Option<Entity>,
    pub status_text: Option<Entity>,
    pub wave_announce_text: Option<Entity>,
    pub lives_bar: Option<Entity>,
    pub lives_bar_bg: Option<Entity>,
    pub tower_select_texts: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EntityHandle(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Tower {
    pub tower_type: TowerType,
    pub cooldown: f32,
    pub target: Option<freecs::Entity>,
    pub fire_animation: f32,
    pub tracking_time: f32,
    pub target_line: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GridCell {
    pub x: i32,
    pub z: i32,
    pub occupied: bool,
    pub is_path: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RangeIndicator {
    pub tower_entity: freecs::Entity,
    pub line_entities: Vec<Entity>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MoneyPopup {
    pub text_entity: Entity,
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
    pub target: freecs::Entity,
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
