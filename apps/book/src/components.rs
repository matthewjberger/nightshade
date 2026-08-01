use nightshade::prelude::Vec3;

#[derive(Clone, Copy, Debug, Default)]
pub struct DemoOwned(pub &'static str);

#[derive(Clone, Copy, Debug)]
pub struct SpinAxis(pub Vec3);

impl Default for SpinAxis {
    fn default() -> Self {
        Self(Vec3::y())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SpinRate(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct OrbitRadius(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct OrbitHeight(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct OrbitRate(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct OrbitPhase(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Focus(pub Vec3);

#[derive(Clone, Copy, Debug, Default)]
pub struct Yaw(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Pitch(pub f32);

#[derive(Clone, Copy, Debug)]
pub struct Distance(pub f32);

impl Default for Distance {
    fn default() -> Self {
        Self(4.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DistanceLimits(pub f32, pub f32);

impl Default for DistanceLimits {
    fn default() -> Self {
        Self(1.5, 24.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PickRadius(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Slot(pub usize);

#[derive(Clone, Copy, Debug, Default)]
pub struct Picked(pub bool);

#[derive(Clone, Debug, Default)]
pub struct RailLink {
    pub anchor: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct PageLink {
    pub target: String,
}
