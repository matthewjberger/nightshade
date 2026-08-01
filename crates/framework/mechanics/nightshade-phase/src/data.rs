use nalgebra_glm::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct PhaseLook {
    pub fog_color: [f32; 3],
    pub fog_start: f32,
    pub fog_end: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub light_color: Vec3,
}

impl Default for PhaseLook {
    fn default() -> Self {
        Self {
            fog_color: [0.5, 0.5, 0.55],
            fog_start: 10.0,
            fog_end: 80.0,
            saturation: 1.0,
            contrast: 1.0,
            light_color: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhaseTurned {
    pub at: usize,
}
