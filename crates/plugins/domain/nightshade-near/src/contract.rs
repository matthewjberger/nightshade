use nalgebra_glm::Vec3;
use nightshade_ecs::Entity;

pub struct Bulk(pub f32);

impl Default for Bulk {
    fn default() -> Self {
        Self(0.5)
    }
}

pub struct Crowded(pub f32);

impl Default for Crowded {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ask {
    pub tag: u32,
    pub point: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Answer {
    pub tag: u32,
    pub count: u32,
    pub nearest: Option<Entity>,
    pub reach: f32,
}

#[derive(Default)]
pub struct NearFrame {
    pub points: Vec<(Entity, Vec3, f32, f32)>,
}

#[derive(Default)]
pub struct NearAsks {
    pub asks: Vec<Ask>,
}

#[derive(Default)]
pub struct NearAnswers {
    pub answers: Vec<Answer>,
    pub pushes: Vec<(Entity, Vec3)>,
}

pub struct NearSettings {
    pub cell_size: f32,
    pub room: f32,
    pub cell_capacity: u32,
}

impl Default for NearSettings {
    fn default() -> Self {
        Self {
            cell_size: 4.0,
            room: 1.5,
            cell_capacity: 32,
        }
    }
}
