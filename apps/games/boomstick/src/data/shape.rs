use nightshade::prelude::Vec3;

pub struct Part {
    pub at: Vec3,
    pub size: Vec3,
    pub tint: [f32; 4],
    pub glow: [f32; 3],
}
