use nalgebra_glm::Vec3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Orbit(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Drift(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bob(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Height(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spin(pub f32);

#[derive(Clone, Copy, Debug)]
pub struct Axis(pub Vec3);

impl Default for Axis {
    fn default() -> Self {
        Self(Vec3::new(0.0, 1.0, 0.0))
    }
}
