use nalgebra_glm::Vec4;

#[derive(Clone, Copy, Debug)]
pub struct Wash {
    pub color: Vec4,
    pub amount: f32,
    pub fade: f32,
}
