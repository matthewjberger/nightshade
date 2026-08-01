use crate::data::{ExplosionType, ShowEvent, Stats};
use nightshade::prelude::*;
use std::collections::HashMap;

pub struct Launcher {
    pub next_launch_time: f32,
    pub auto: bool,
    pub interval: f32,
    pub salvo: bool,
    pub salvo_count: u32,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            next_launch_time: 0.0,
            auto: true,
            interval: 0.3,
            salvo: false,
            salvo_count: 5,
        }
    }
}

#[derive(Default)]
pub struct Show {
    pub active: bool,
    pub start_time: f32,
    pub events: Vec<ShowEvent>,
    pub index: usize,
}

pub struct Toggles {
    pub fire: bool,
    pub smoke: bool,
    pub grid: bool,
}

impl Default for Toggles {
    fn default() -> Self {
        Self {
            fire: true,
            smoke: true,
            grid: true,
        }
    }
}

#[derive(Default)]
pub struct Interface {
    pub auto_launch: Entity,
    pub launch_interval: Entity,
    pub salvo_mode: Entity,
    pub salvo_count_label: Entity,
    pub salvo_count: Entity,
    pub bloom_intensity: Entity,
    pub show_grid: Entity,
    pub launch_single: Entity,
    pub launch_salvo: Entity,
    pub finale: Entity,
    pub show_play: Entity,
    pub show_status_label: Entity,
    pub shells_label: Entity,
    pub sub_shells_label: Entity,
    pub fire_label: Entity,
    pub sprite_label: Entity,
    pub smoke_label: Entity,
    pub emitters_label: Entity,
    pub type_buttons: HashMap<Entity, ExplosionType>,
    pub last_show_active: bool,
    pub last_salvo_mode: bool,
    pub last_stats: Option<Stats>,
}
