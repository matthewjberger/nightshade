use nalgebra_glm::{Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct ParticleTextureUpload {
    pub slot: u32,
    pub rgba_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum EmitterType {
    Firework,
    Fire,
    Smoke,
    Sparks,
    Trail,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    enum2schema::Schema,
)]
pub enum EmitterShape {
    #[default]
    Point,
    Sphere {
        radius: f32,
    },
    Cone {
        angle: f32,
        height: f32,
    },
    Box {
        half_extents: Vec3,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
pub struct ColorGradient {
    pub colors: Vec<(f32, Vec4)>,
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (1.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            ],
        }
    }
}

impl ColorGradient {
    pub fn firework_trail() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.95, 1.0)),
                (0.15, Vec4::new(1.0, 0.95, 0.7, 1.0)),
                (0.4, Vec4::new(1.0, 0.7, 0.3, 0.85)),
                (0.7, Vec4::new(1.0, 0.4, 0.1, 0.5)),
                (1.0, Vec4::new(0.6, 0.15, 0.0, 0.0)),
            ],
        }
    }

    pub fn firework_explosion(base_color: Vec3) -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (
                    0.05,
                    Vec4::new(
                        (base_color.x + 1.0) * 0.5,
                        (base_color.y + 1.0) * 0.5,
                        (base_color.z + 1.0) * 0.5,
                        1.0,
                    ),
                ),
                (
                    0.15,
                    Vec4::new(base_color.x, base_color.y, base_color.z, 1.0),
                ),
                (
                    0.4,
                    Vec4::new(base_color.x, base_color.y, base_color.z, 0.95),
                ),
                (
                    0.65,
                    Vec4::new(
                        base_color.x * 0.85,
                        base_color.y * 0.85,
                        base_color.z * 0.85,
                        0.7,
                    ),
                ),
                (
                    0.85,
                    Vec4::new(
                        base_color.x * 0.5,
                        base_color.y * 0.5,
                        base_color.z * 0.5,
                        0.3,
                    ),
                ),
                (1.0, Vec4::new(0.1, 0.1, 0.1, 0.0)),
            ],
        }
    }

    pub fn firework_willow(base_color: Vec3) -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.95, 1.0)),
                (
                    0.1,
                    Vec4::new(base_color.x, base_color.y, base_color.z, 1.0),
                ),
                (
                    0.3,
                    Vec4::new(base_color.x, base_color.y, base_color.z, 0.9),
                ),
                (
                    0.6,
                    Vec4::new(
                        base_color.x * 0.7,
                        base_color.y * 0.8,
                        base_color.z * 0.6,
                        0.6,
                    ),
                ),
                (
                    0.85,
                    Vec4::new(
                        base_color.x * 0.4,
                        base_color.y * 0.5,
                        base_color.z * 0.3,
                        0.25,
                    ),
                ),
                (1.0, Vec4::new(0.1, 0.1, 0.05, 0.0)),
            ],
        }
    }

    pub fn firework_glitter() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.2, Vec4::new(1.0, 1.0, 0.9, 1.0)),
                (0.5, Vec4::new(1.0, 0.95, 0.7, 0.9)),
                (0.8, Vec4::new(1.0, 0.85, 0.5, 0.5)),
                (1.0, Vec4::new(0.8, 0.6, 0.2, 0.0)),
            ],
        }
    }

    pub fn firework_crackle() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.1, Vec4::new(1.0, 0.95, 0.8, 1.0)),
                (0.3, Vec4::new(1.0, 0.8, 0.4, 0.9)),
                (0.6, Vec4::new(1.0, 0.5, 0.1, 0.6)),
                (1.0, Vec4::new(0.4, 0.1, 0.0, 0.0)),
            ],
        }
    }

    pub fn flash_burst() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.15, Vec4::new(1.0, 1.0, 0.95, 1.0)),
                (0.4, Vec4::new(1.0, 0.95, 0.8, 0.7)),
                (0.7, Vec4::new(1.0, 0.9, 0.6, 0.3)),
                (1.0, Vec4::new(1.0, 0.8, 0.4, 0.0)),
            ],
        }
    }

    pub fn comet_trail() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.1, Vec4::new(1.0, 0.98, 0.9, 1.0)),
                (0.25, Vec4::new(1.0, 0.9, 0.6, 0.95)),
                (0.5, Vec4::new(1.0, 0.7, 0.3, 0.8)),
                (0.75, Vec4::new(1.0, 0.4, 0.1, 0.5)),
                (1.0, Vec4::new(0.5, 0.1, 0.0, 0.0)),
            ],
        }
    }

    pub fn palm_frond(base_color: Vec3) -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (
                    0.08,
                    Vec4::new(base_color.x, base_color.y, base_color.z, 1.0),
                ),
                (
                    0.25,
                    Vec4::new(base_color.x, base_color.y, base_color.z, 0.95),
                ),
                (
                    0.5,
                    Vec4::new(
                        base_color.x * 0.9,
                        base_color.y * 0.9,
                        base_color.z * 0.9,
                        0.85,
                    ),
                ),
                (
                    0.75,
                    Vec4::new(
                        base_color.x * 0.7,
                        base_color.y * 0.7,
                        base_color.z * 0.7,
                        0.5,
                    ),
                ),
                (
                    1.0,
                    Vec4::new(
                        base_color.x * 0.3,
                        base_color.y * 0.3,
                        base_color.z * 0.3,
                        0.0,
                    ),
                ),
            ],
        }
    }

    pub fn crossette() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.1, Vec4::new(1.0, 0.95, 0.85, 1.0)),
                (0.3, Vec4::new(1.0, 0.85, 0.6, 0.9)),
                (0.6, Vec4::new(1.0, 0.6, 0.2, 0.7)),
                (1.0, Vec4::new(0.6, 0.2, 0.0, 0.0)),
            ],
        }
    }

    pub fn strobe() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.5, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (1.0, Vec4::new(0.8, 0.8, 0.8, 0.0)),
            ],
        }
    }

    pub fn fire() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.9, 1.0)),
                (0.2, Vec4::new(1.0, 0.9, 0.4, 1.0)),
                (0.4, Vec4::new(1.0, 0.5, 0.1, 0.9)),
                (0.7, Vec4::new(0.9, 0.2, 0.0, 0.6)),
                (1.0, Vec4::new(0.3, 0.05, 0.0, 0.0)),
            ],
        }
    }

    pub fn smoke() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(0.4, 0.4, 0.45, 0.0)),
                (0.1, Vec4::new(0.5, 0.5, 0.55, 0.6)),
                (0.4, Vec4::new(0.55, 0.55, 0.6, 0.5)),
                (0.7, Vec4::new(0.6, 0.6, 0.65, 0.3)),
                (1.0, Vec4::new(0.65, 0.65, 0.7, 0.0)),
            ],
        }
    }

    pub fn sparks() -> Self {
        Self {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.8, 1.0)),
                (0.3, Vec4::new(1.0, 0.8, 0.2, 1.0)),
                (0.7, Vec4::new(1.0, 0.4, 0.0, 0.7)),
                (1.0, Vec4::new(0.5, 0.1, 0.0, 0.0)),
            ],
        }
    }

    pub fn to_gpu_data(&self) -> [[f32; 8]; 8] {
        let mut data = [[0.0f32; 8]; 8];
        for (index, (time, color)) in self.colors.iter().take(8).enumerate() {
            data[index] = [*time, color.x, color.y, color.z, color.w, 0.0, 0.0, 0.0];
        }
        data
    }

    pub fn color_count(&self) -> u32 {
        self.colors.len().min(8) as u32
    }
}

pub fn emitter_shape_code(shape: &EmitterShape) -> (u32, [f32; 4]) {
    match shape {
        EmitterShape::Point => (0, [0.0; 4]),
        EmitterShape::Sphere { radius } => (1, [*radius, 0.0, 0.0, 0.0]),
        EmitterShape::Cone { angle, height } => (2, [*angle, *height, 0.0, 0.0]),
        EmitterShape::Box { half_extents } => {
            (3, [half_extents.x, half_extents.y, half_extents.z, 0.0])
        }
    }
}

pub fn emitter_type_code(emitter_type: EmitterType) -> u32 {
    match emitter_type {
        EmitterType::Firework => 0,
        EmitterType::Fire => 1,
        EmitterType::Smoke => 2,
        EmitterType::Sparks => 3,
        EmitterType::Trail => 4,
    }
}

pub fn pack_color_gradient(gradient: &ColorGradient) -> ([[f32; 4]; 16], u32) {
    let mut packed = [[0.0f32; 4]; 16];
    let stops = gradient.colors.len().min(8);
    for (slot, (time, color)) in gradient.colors.iter().take(stops).enumerate() {
        packed[slot * 2] = [*time, color.x, color.y, color.z];
        packed[slot * 2 + 1] = [color.w, 0.0, 0.0, 0.0];
    }
    (packed, stops as u32)
}

pub fn pack_value_curve(points: &[(f32, f32)]) -> ([[f32; 4]; 8], u32) {
    let mut packed = [[0.0f32; 4]; 8];
    let count = points.len().min(16);
    for (index, (at, value)) in points.iter().take(count).enumerate() {
        let slot = &mut packed[index / 2];
        let lane = (index % 2) * 2;
        slot[lane] = *at;
        slot[lane + 1] = *value;
    }
    (packed, count as u32)
}
