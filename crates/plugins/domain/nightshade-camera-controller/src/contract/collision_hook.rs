use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CameraSweep {
    pub focus: Vec3,
    pub ideal_position: Vec3,
    pub radius: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PendingCameraSweep {
    pub ask: CameraSweep,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SweptDistance {
    pub distance: f32,
}
