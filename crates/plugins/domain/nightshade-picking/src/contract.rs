use nalgebra_glm::Vec3;
use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy)]
pub struct PickingRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl PickingRay {
    pub fn intersect_plane(&self, plane_normal: Vec3, plane_distance: f32) -> Option<Vec3> {
        let denom = nalgebra_glm::dot(&plane_normal, &self.direction);
        if denom.abs() < 1e-6 {
            return None;
        }

        let t = -(nalgebra_glm::dot(&plane_normal, &self.origin) + plane_distance) / denom;
        if t < 0.0 {
            return None;
        }

        Some(self.origin + self.direction * t)
    }

    pub fn intersect_ground_plane(&self, y_level: f32) -> Option<Vec3> {
        self.intersect_plane(Vec3::new(0.0, 1.0, 0.0), -y_level)
    }
}

#[derive(Debug, Clone)]
pub struct PickingResult {
    pub entity: Entity,
    pub distance: f32,
    pub world_position: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct PickingOptions {
    pub max_distance: f32,
    pub ignore_invisible: bool,
}

impl Default for PickingOptions {
    fn default() -> Self {
        Self {
            max_distance: f32::INFINITY,
            ignore_invisible: true,
        }
    }
}
