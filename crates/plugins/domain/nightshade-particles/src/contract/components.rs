use nalgebra_glm::Vec3;

use crate::contract::data::{ColorGradient, EmitterShape, EmitterType};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
pub struct ParticleEmitter {
    pub emitter_type: EmitterType,
    pub shape: EmitterShape,
    pub position: Vec3,
    pub direction: Vec3,
    pub spawn_rate: f32,
    pub burst_count: u32,
    pub particle_lifetime_min: f32,
    pub particle_lifetime_max: f32,
    pub initial_velocity_min: f32,
    pub initial_velocity_max: f32,
    pub velocity_spread: f32,
    pub gravity: Vec3,
    pub drag: f32,
    pub size_start: f32,
    pub size_end: f32,
    pub color_gradient: ColorGradient,
    pub emissive_strength: f32,
    pub enabled: bool,
    pub accumulated_spawn: f32,
    pub one_shot: bool,
    pub has_fired: bool,
    pub turbulence_strength: f32,
    pub turbulence_frequency: f32,
    pub texture_index: u32,
    #[serde(default)]
    pub size_curve: Vec<(f32, f32)>,
    #[serde(default)]
    pub opacity_curve: Vec<(f32, f32)>,
    #[serde(default)]
    pub death_sub_emitter: Option<String>,
    #[serde(default)]
    pub trail_sub_emitter: Option<String>,
    #[serde(default)]
    pub trail_spawn_interval: f32,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Point,
            position: Vec3::zeros(),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 100.0,
            burst_count: 0,
            particle_lifetime_min: 1.0,
            particle_lifetime_max: 2.0,
            initial_velocity_min: 1.0,
            initial_velocity_max: 3.0,
            velocity_spread: 0.3,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.1,
            size_start: 0.1,
            size_end: 0.0,
            color_gradient: ColorGradient::default(),
            emissive_strength: 1.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.0,
            turbulence_frequency: 1.0,
            texture_index: 0,
            size_curve: Vec::new(),
            opacity_curve: Vec::new(),
            death_sub_emitter: None,
            trail_sub_emitter: None,
            trail_spawn_interval: 0.0,
        }
    }
}
