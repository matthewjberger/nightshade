use nalgebra_glm::{Vec2, Vec3};

#[derive(Clone, Debug, Default)]
pub struct TextMesh {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
    pub bounds: Vec2,
}

pub const GLYPH_COLORED: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub character_index: u32,
    pub flags: u32,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum FontKind {
    #[default]
    Default,
    Mono,
    IconsMaterial,
    IconsLucide,
}

#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
    Baseline,
}
