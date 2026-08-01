#[derive(Clone, Copy)]
pub struct FlickerBase(pub f32);

impl Default for FlickerBase {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy)]
pub struct Flickering(pub f32);

impl Default for Flickering {
    fn default() -> Self {
        Self(1.0)
    }
}
