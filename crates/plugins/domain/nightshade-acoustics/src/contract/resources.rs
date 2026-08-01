use crate::contract::data::{
    EchoPoint, EmitterPlace, HIGH_BAND_HERTZ, LOW_BAND_HERTZ, MATERIALS, MAX_MATERIALS,
    MaterialAcoustics, RayStep, SoundPath,
};
use nalgebra_glm::Vec3;
use nightshade_ecs::Entity;

pub struct AcousticsGrid {
    pub origin: Vec3,
    pub cell: f32,
    pub side: u32,
    pub cells: Vec<u8>,
    pub materials: [MaterialAcoustics; MAX_MATERIALS],
    pub revision: u64,
}

impl Default for AcousticsGrid {
    fn default() -> Self {
        let side = 64;
        Self {
            origin: Vec3::new(-32.0, -8.0, -32.0),
            cell: 1.0,
            side,
            cells: vec![0; (side as usize).pow(3)],
            materials: MATERIALS,
            revision: 0,
        }
    }
}

#[derive(Default)]
pub struct AcousticsRoom {
    pub heard: RoomReading,
    pub zones: Vec<(Entity, RoomReading)>,
}

pub struct AcousticsSettings {
    pub reverb_rays: u32,
    pub reverb_bounces: u32,
    pub occlusion_rays: u32,
    pub occlusion_bounces: u32,
    pub permeation_rays: u32,
    pub permeation_bounces: u32,
    pub ambient_occlusion_rays: u32,
    pub ambient_occlusion_bounces: u32,
    pub ambient_permeation_rays: u32,
    pub ambient_permeation_bounces: u32,
    pub reach: f32,
    pub sweep_slices: u32,
    pub refresh_distance: f32,
    pub speed_of_sound: f32,
    pub metres_per_unit: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub occlusion_cap: f32,
    pub permeation_cap: f32,
    pub reverb_cap: f32,
    pub ambience_cap: f32,
    pub loudest: f32,
    pub zones: u32,
    pub zone_rays: u32,
    pub visualisation_rays: u32,
    pub relative_inner: f32,
    pub relative_outer: f32,
    pub low_band: f32,
    pub high_band: f32,
    pub reverb_only: bool,
    pub outside_is_muffled: bool,
    pub indoors: bool,
    pub epsilon: f32,
}

impl Default for AcousticsSettings {
    fn default() -> Self {
        Self {
            reverb_rays: 2048,
            reverb_bounces: 8,
            occlusion_rays: 1024,
            occlusion_bounces: 6,
            permeation_rays: 256,
            permeation_bounces: 3,
            ambient_occlusion_rays: 512,
            ambient_occlusion_bounces: 6,
            ambient_permeation_rays: 256,
            ambient_permeation_bounces: 3,
            reach: 60.0,
            sweep_slices: 3,
            refresh_distance: 1.0,
            speed_of_sound: 343.0,
            metres_per_unit: 1.0,
            temperature: 26.0,
            humidity: 0.1,
            pressure: 101_325.0,
            occlusion_cap: 0.15,
            permeation_cap: 0.15,
            reverb_cap: 0.2,
            ambience_cap: 0.15,
            loudest: 1.0,
            zones: 3,
            zone_rays: 512,
            visualisation_rays: 24,
            relative_inner: 0.6,
            relative_outer: 0.8,
            low_band: LOW_BAND_HERTZ,
            high_band: HIGH_BAND_HERTZ,
            reverb_only: false,
            outside_is_muffled: true,
            indoors: false,
            epsilon: 0.01,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RoomReading {
    pub loudness: f32,
    pub span: f32,
    pub outdoors: f32,
    pub outdoor_low: f32,
    pub outdoor_high: f32,
    pub absorbed: f32,
    pub scattered: f32,
    pub decay: f32,
    pub decay_high: f32,
    pub early_gain: f32,
    pub early_delay: f32,
    pub late_gain: f32,
    pub late_delay: f32,
    pub early_from: Vec3,
    pub late_from: Vec3,
    pub opening: Vec3,
}

impl Default for RoomReading {
    fn default() -> Self {
        Self {
            loudness: 0.0,
            span: 0.0,
            outdoors: 1.0,
            outdoor_low: 1.0,
            outdoor_high: 1.0,
            absorbed: 0.1,
            scattered: 0.1,
            decay: 1.0,
            decay_high: 0.7,
            early_gain: 0.0,
            early_delay: 0.007,
            late_gain: 0.0,
            late_delay: 0.011,
            early_from: Vec3::zeros(),
            late_from: Vec3::zeros(),
            opening: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

#[derive(Default)]
pub struct AcousticsFrame {
    pub leeches: Vec<(Entity, Entity)>,
    pub listener: Vec3,
    pub heard: bool,
    pub emitters: Vec<EmitterPlace>,
}

#[derive(Default)]
pub struct AcousticsHeard {
    pub paths: Vec<(Entity, SoundPath)>,
}

#[derive(Default)]
pub struct AcousticsEcho {
    pub points: Vec<EchoPoint>,
    pub revision: u64,
}

#[derive(Default)]
pub struct AcousticsRays {
    pub steps: Vec<RayStep>,
    pub revision: u64,
    pub span: f32,
}
