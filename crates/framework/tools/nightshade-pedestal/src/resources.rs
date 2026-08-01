#[derive(Clone, Copy)]
pub struct Pedestal {
    pub model: &'static str,
    pub sky: &'static [u8],
    pub back: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub compressed: bool,
}

impl Default for Pedestal {
    fn default() -> Self {
        Self {
            model: "Box",
            sky: &[],
            back: 2.6,
            yaw: 0.0,
            pitch: 0.08,
            compressed: false,
        }
    }
}

#[derive(Default)]
pub struct PendingModel {
    pub handle: Option<nightshade_load::LoadHandle>,
}
