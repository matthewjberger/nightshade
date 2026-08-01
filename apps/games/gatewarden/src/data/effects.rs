use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Puff {
    Sparks,
    Glow,
    Smoke,
}

pub struct BurstStyle {
    pub puff: Puff,
    pub count: u32,
    pub radius: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub spread: f32,
    pub rise: f32,
    pub drag: f32,
    pub size_start: f32,
    pub size_end: f32,
    pub life_min: f32,
    pub life_max: f32,
    pub emissive: f32,
    pub colors: [Vec4; 3],
}

pub fn hit_style(amount: i32) -> BurstStyle {
    let weight = amount.max(0) as f32;
    BurstStyle {
        puff: Puff::Sparks,
        count: (34 + amount.max(0) as u32 * 12).min(150),
        radius: 0.12,
        speed_min: 2.6,
        speed_max: 6.5 + weight * 0.2,
        spread: 1.0,
        rise: -5.0,
        drag: 0.1,
        size_start: 0.07 + weight * 0.006,
        size_end: 0.01,
        life_min: 0.35,
        life_max: 0.9,
        emissive: 9.0,
        colors: [
            Vec4::new(1.0, 0.9, 0.5, 1.0),
            Vec4::new(1.0, 0.45, 0.15, 1.0),
            Vec4::new(0.9, 0.15, 0.1, 0.0),
        ],
    }
}

pub fn shield_style() -> BurstStyle {
    BurstStyle {
        puff: Puff::Glow,
        count: 56,
        radius: 0.42,
        speed_min: 0.2,
        speed_max: 0.85,
        spread: 0.18,
        rise: 0.5,
        drag: 2.0,
        size_start: 0.13,
        size_end: 0.03,
        life_min: 0.7,
        life_max: 1.25,
        emissive: 10.0,
        colors: [
            Vec4::new(0.85, 0.98, 1.0, 1.0),
            Vec4::new(0.4, 0.8, 1.0, 0.9),
            Vec4::new(0.2, 0.5, 0.95, 0.0),
        ],
    }
}

pub fn block_style() -> BurstStyle {
    BurstStyle {
        puff: Puff::Glow,
        count: 44,
        radius: 0.4,
        speed_min: 1.6,
        speed_max: 4.2,
        spread: 0.4,
        rise: 0.0,
        drag: 1.5,
        size_start: 0.1,
        size_end: 0.02,
        life_min: 0.3,
        life_max: 0.6,
        emissive: 13.0,
        colors: [
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            Vec4::new(0.55, 0.9, 1.0, 0.9),
            Vec4::new(0.25, 0.55, 1.0, 0.0),
        ],
    }
}

pub fn heal_style() -> BurstStyle {
    BurstStyle {
        puff: Puff::Glow,
        count: 46,
        radius: 0.3,
        speed_min: 0.4,
        speed_max: 1.5,
        spread: 0.3,
        rise: 2.4,
        drag: 0.7,
        size_start: 0.09,
        size_end: 0.02,
        life_min: 0.8,
        life_max: 1.4,
        emissive: 7.0,
        colors: [
            Vec4::new(0.85, 1.0, 0.88, 1.0),
            Vec4::new(0.35, 0.95, 0.55, 0.9),
            Vec4::new(0.5, 1.0, 0.7, 0.0),
        ],
    }
}

pub fn death_style() -> BurstStyle {
    BurstStyle {
        puff: Puff::Smoke,
        count: 32,
        radius: 0.26,
        speed_min: 0.5,
        speed_max: 1.6,
        spread: 0.7,
        rise: 0.9,
        drag: 1.7,
        size_start: 0.1,
        size_end: 0.34,
        life_min: 0.9,
        life_max: 1.6,
        emissive: 0.6,
        colors: [
            Vec4::new(0.6, 0.58, 0.6, 0.85),
            Vec4::new(0.4, 0.38, 0.42, 0.5),
            Vec4::new(0.25, 0.24, 0.28, 0.0),
        ],
    }
}

pub fn victory_style() -> BurstStyle {
    BurstStyle {
        puff: Puff::Glow,
        count: 200,
        radius: 0.2,
        speed_min: 2.5,
        speed_max: 7.0,
        spread: 1.0,
        rise: -3.0,
        drag: 0.2,
        size_start: 0.13,
        size_end: 0.02,
        life_min: 0.9,
        life_max: 1.8,
        emissive: 12.0,
        colors: [
            Vec4::new(1.0, 0.96, 0.7, 1.0),
            Vec4::new(1.0, 0.78, 0.25, 1.0),
            Vec4::new(1.0, 0.4, 0.1, 0.0),
        ],
    }
}

pub fn defeat_style() -> BurstStyle {
    BurstStyle {
        puff: Puff::Smoke,
        count: 96,
        radius: 0.34,
        speed_min: 0.9,
        speed_max: 2.6,
        spread: 0.9,
        rise: 0.7,
        drag: 1.5,
        size_start: 0.14,
        size_end: 0.42,
        life_min: 1.0,
        life_max: 1.9,
        emissive: 0.5,
        colors: [
            Vec4::new(0.55, 0.3, 0.3, 0.9),
            Vec4::new(0.35, 0.2, 0.22, 0.55),
            Vec4::new(0.2, 0.16, 0.18, 0.0),
        ],
    }
}
