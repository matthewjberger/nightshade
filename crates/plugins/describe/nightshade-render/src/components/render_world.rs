use nalgebra_glm::Mat4;

#[derive(Clone, Copy, PartialEq)]
pub struct Transform(pub Mat4);

impl Default for Transform {
    fn default() -> Self {
        Self(Mat4::identity())
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct MeshName(pub String);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct MeshKey(pub u64);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct MaterialId(pub u32);

#[derive(Clone, Copy, PartialEq)]
pub struct Visible(pub bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct MorphWeights(pub [f32; 8]);

#[derive(Clone, Copy, PartialEq)]
pub struct CullingMask(pub u32);

impl Default for CullingMask {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Layer(pub u32);

pub struct ShadowCaster;

pub struct Outlined;
