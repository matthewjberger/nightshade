use nightshade::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TowerType {
    #[default]
    Basic,
    Frost,
    Cannon,
    Sniper,
    Poison,
}

pub struct TowerRow {
    pub cost: u32,
    pub damage: f32,
    pub range: f32,
    pub fire_rate: f32,
    pub projectile_speed: f32,
    pub color: [f32; 4],
    pub name: &'static str,
    pub description: &'static str,
    pub hotkey_label: &'static str,
}

pub const TOWER_KINDS: [TowerType; 5] = [
    TowerType::Basic,
    TowerType::Frost,
    TowerType::Cannon,
    TowerType::Sniper,
    TowerType::Poison,
];

const TOWER_ROWS: [TowerRow; 5] = [
    TowerRow {
        cost: 60,
        damage: 15.0,
        range: 3.0,
        fire_rate: 0.5,
        projectile_speed: 12.0,
        color: [1.0, 0.5, 0.0, 1.0],
        name: "Basic",
        description: "Balanced fire",
        hotkey_label: "1",
    },
    TowerRow {
        cost: 120,
        damage: 8.0,
        range: 2.5,
        fire_rate: 1.0,
        projectile_speed: 8.0,
        color: [0.2, 0.6, 1.0, 1.0],
        name: "Frost",
        description: "Slows on hit",
        hotkey_label: "2",
    },
    TowerRow {
        cost: 200,
        damage: 50.0,
        range: 4.0,
        fire_rate: 2.0,
        projectile_speed: 10.0,
        color: [0.8, 0.2, 0.2, 1.0],
        name: "Cannon",
        description: "Splash damage",
        hotkey_label: "3",
    },
    TowerRow {
        cost: 180,
        damage: 80.0,
        range: 6.0,
        fire_rate: 3.0,
        projectile_speed: 20.0,
        color: [0.3, 0.3, 0.3, 1.0],
        name: "Sniper",
        description: "Charged shot",
        hotkey_label: "4",
    },
    TowerRow {
        cost: 150,
        damage: 5.0,
        range: 2.8,
        fire_rate: 0.8,
        projectile_speed: 10.0,
        color: [0.6, 0.2, 0.8, 1.0],
        name: "Poison",
        description: "Damage over time",
        hotkey_label: "5",
    },
];

pub fn tower(tower_type: TowerType) -> &'static TowerRow {
    &TOWER_ROWS[tower_type as usize]
}

pub fn tower_color(tower_type: TowerType) -> Vec4 {
    let channels = tower(tower_type).color;
    nalgebra_glm::vec4(channels[0], channels[1], channels[2], channels[3])
}
