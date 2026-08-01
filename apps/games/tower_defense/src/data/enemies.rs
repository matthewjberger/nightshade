use nightshade::prelude::*;

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

pub struct EnemyRow {
    pub base_health: f32,
    pub speed: f32,
    pub base_value: u32,
    pub shield: f32,
    pub color: [f32; 4],
    pub scale: [f32; 3],
    pub flies: bool,
}

const ENEMY_ROWS: [EnemyRow; 7] = [
    EnemyRow {
        base_health: 50.0,
        speed: 2.0,
        base_value: 10,
        shield: 0.0,
        color: [0.8, 0.2, 0.2, 1.0],
        scale: [0.4, 0.6, 0.4],
        flies: false,
    },
    EnemyRow {
        base_health: 30.0,
        speed: 4.0,
        base_value: 15,
        shield: 0.0,
        color: [1.0, 0.5, 0.0, 1.0],
        scale: [0.3, 0.5, 0.3],
        flies: false,
    },
    EnemyRow {
        base_health: 150.0,
        speed: 1.0,
        base_value: 30,
        shield: 0.0,
        color: [0.4, 0.4, 0.4, 1.0],
        scale: [0.6, 0.6, 0.6],
        flies: false,
    },
    EnemyRow {
        base_health: 40.0,
        speed: 2.5,
        base_value: 20,
        shield: 0.0,
        color: [0.5, 0.8, 1.0, 1.0],
        scale: [0.5, 0.5, 0.5],
        flies: true,
    },
    EnemyRow {
        base_health: 60.0,
        speed: 1.5,
        base_value: 25,
        shield: 30.0,
        color: [0.2, 0.6, 0.9, 1.0],
        scale: [0.5, 0.5, 0.5],
        flies: false,
    },
    EnemyRow {
        base_health: 80.0,
        speed: 1.8,
        base_value: 35,
        shield: 0.0,
        color: [0.2, 0.9, 0.4, 1.0],
        scale: [0.4, 0.4, 0.4],
        flies: false,
    },
    EnemyRow {
        base_health: 500.0,
        speed: 0.8,
        base_value: 100,
        shield: 100.0,
        color: [0.6, 0.0, 0.6, 1.0],
        scale: [1.0, 1.0, 1.0],
        flies: false,
    },
];

pub fn enemy(enemy_type: EnemyType) -> &'static EnemyRow {
    &ENEMY_ROWS[enemy_type as usize]
}

pub fn enemy_health(enemy_type: EnemyType, wave: u32) -> f32 {
    let health_multiplier = 1.0 + (wave as f32 - 1.0) * 0.5;
    enemy(enemy_type).base_health * health_multiplier
}

pub fn enemy_value(enemy_type: EnemyType, wave: u32) -> u32 {
    enemy(enemy_type).base_value + wave * 2
}

pub fn enemy_color(enemy_type: EnemyType) -> Vec4 {
    let channels = enemy(enemy_type).color;
    nalgebra_glm::vec4(channels[0], channels[1], channels[2], channels[3])
}

pub fn enemy_y_offset(enemy_type: EnemyType) -> f32 {
    let row = enemy(enemy_type);
    let base_offset = row.scale[1] / 2.0 - 0.5;
    if row.flies {
        2.0 + base_offset
    } else {
        base_offset
    }
}
