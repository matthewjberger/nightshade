use nightshade::prelude::Vec3;

pub struct Picture;

pub const MODELS: [(&str, Vec3); 8] = [
    ("DamagedHelmet", Vec3::new(-6.0, 1.2, 0.0)),
    ("BoomBox", Vec3::new(-4.0, 0.6, 0.0)),
    ("Fox", Vec3::new(-2.0, 0.0, 0.0)),
    ("SheenChair", Vec3::new(0.0, 0.0, 0.0)),
    ("MetalRoughSpheres", Vec3::new(2.0, 0.6, 0.0)),
    ("ClearCoatTest", Vec3::new(4.0, 0.6, 0.0)),
    ("CompareIridescence", Vec3::new(6.0, 0.6, 0.0)),
    ("DragonDispersion", Vec3::new(8.0, 0.6, 0.0)),
];

pub const PICTURES: usize = 1;
pub const SOUNDS: usize = 1;

pub const TOTAL: usize = MODELS.len() + PICTURES + SOUNDS;
