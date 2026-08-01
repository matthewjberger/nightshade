use nalgebra_glm::{Mat4, Vec3};
use nightshade_ecs::Entity;

#[derive(Clone, Copy, Debug)]
pub struct PortalPlane {
    pub center: Vec3,
    pub normal: Vec3,
    pub up: Vec3,
    pub width: f32,
    pub height: f32,
}

impl Default for PortalPlane {
    fn default() -> Self {
        Self {
            center: Vec3::zeros(),
            normal: Vec3::z(),
            up: Vec3::y(),
            width: 1.0,
            height: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PortalOrder {
    pub camera: Entity,
    pub source: Entity,
    pub model: Mat4,
    pub rim: [f32; 4],
}

#[derive(Default)]
pub struct PortalFrame {
    pub elapsed: f32,
    pub orders: Vec<PortalOrder>,
}
