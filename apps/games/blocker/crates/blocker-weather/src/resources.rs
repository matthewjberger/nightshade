use crate::data::weather::WeatherKind;
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::Entity;

pub struct Sky {
    pub kind: WeatherKind,
    pub told: Option<WeatherKind>,
    pub until: f32,
    pub rain: f32,
    pub snow: f32,
    pub cloud: f32,
    pub fog: f32,
    pub wind: f32,
    pub wind_heading: f32,
    pub flash: f32,
    pub until_lightning: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            kind: WeatherKind::Clear,
            told: Some(WeatherKind::Clear),
            until: theme::WEATHER_FIRST,
            rain: 0.0,
            snow: 0.0,
            cloud: 0.0,
            fog: 0.0,
            wind: 0.0,
            wind_heading: 0.0,
            flash: 0.0,
            until_lightning: 0.0,
        }
    }
}

pub struct Season {
    pub elapsed: f32,
    pub share: f32,
    pub temperature: f32,
    pub told: Option<usize>,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            share: 0.25,
            temperature: 0.0,
            told: None,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Overhead(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Settling(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Falling {
    pub rain: Vec<Entity>,
    pub snow: Vec<Entity>,
    pub splash: Vec<Entity>,
    pub anchor: Option<IVec3>,
    pub floors: Vec<(f32, bool)>,
}
