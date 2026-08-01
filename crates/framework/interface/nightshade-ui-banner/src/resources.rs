use nalgebra_glm::Vec4;
use nightshade::ecs::Entity;

pub struct Banner {
    pub line: String,
    pub color: Vec4,
    pub shown: bool,
}

impl Default for Banner {
    fn default() -> Self {
        Self {
            line: String::new(),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            shown: true,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct BannerHandles {
    pub panel: Entity,
    pub text: Entity,
}

#[derive(Default)]
pub struct BannerShown {
    pub line: String,
    pub color: Option<Vec4>,
    pub shown: Option<bool>,
}
