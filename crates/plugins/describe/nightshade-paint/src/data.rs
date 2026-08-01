use nalgebra_glm::{Vec2, Vec4};

use nightshade_text::contract::FontKind;
use nightshade_text::contract::{TextAlignment, TextMesh, VerticalAlignment};

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawShadow {
    pub color: Vec4,
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
}

#[derive(Clone, Debug)]
pub struct DrawRect {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Vec4,
    pub rotation: f32,
    pub clip_rect: Option<Rect>,
    pub layer: u8,
    pub z_index: i32,
    pub shadow: Option<DrawShadow>,
    pub effect_kind: u32,
    pub effect_params: [f32; 4],
    pub quad_corners: Option<[Vec2; 4]>,
}

impl Default for DrawRect {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 100.0),
            color: Vec4::new(0.2, 0.2, 0.2, 0.8),
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            rotation: 0.0,
            clip_rect: None,
            layer: 0,
            z_index: 0,
            shadow: None,
            effect_kind: 0,
            effect_params: [0.0; 4],
            quad_corners: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawImage {
    pub entity: Option<nightshade::ecs::Entity>,
    pub position: Vec2,
    pub size: Vec2,
    pub texture_index: u32,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub color: Vec4,
    pub clip_rect: Option<Rect>,
    pub layer: u8,
    pub z_index: i32,
    pub rotation: f32,
    pub nearest: bool,
}

pub const IMAGE_FLAG_NEAREST: u32 = 1;

#[derive(Clone)]
pub struct DrawText {
    pub entity: Option<nightshade::ecs::Entity>,
    pub text: String,
    pub mesh: TextMesh,
    pub position: Vec2,
    pub color: Vec4,
    pub outline_color: Vec4,
    pub outline_width: f32,
    pub font_size: f32,
    pub alignment: TextAlignment,
    pub vertical_alignment: VerticalAlignment,
    pub monospace: bool,
    pub font_kind: FontKind,
    pub wrap_width: Option<f32>,
    pub clip_rect: Option<Rect>,
    pub layer: u8,
    pub z_index: i32,
    pub rotation: f32,
    pub pivot: Vec2,
    pub character_colors: Option<Vec<Option<Vec4>>>,
    pub character_background_colors: Option<Vec<Option<Vec4>>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + width, y + height),
        }
    }

    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn x(&self) -> f32 {
        self.min.x
    }

    pub fn y(&self) -> f32 {
        self.min.y
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec2::new(amount, amount),
            max: self.max + Vec2::new(amount, amount),
        }
    }

    pub fn shrink(&self, amount: f32) -> Self {
        self.expand(-amount)
    }

    pub fn translate(&self, offset: Vec2) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let min_x = self.min.x.max(other.min.x);
        let min_y = self.min.y.max(other.min.y);
        let max_x = self.max.x.min(other.max.x);
        let max_y = self.max.y.min(other.max.y);

        if min_x < max_x && min_y < max_y {
            Some(Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
