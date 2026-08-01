use nightshade::prelude::{Vec4, vec4};
use nightshade_ui::commands::builder::UiNodeBuilder;
use nightshade_ui::prelude::*;

use crate::queries::code::Token;

pub fn ink_color() -> Vec4 {
    vec4(1.0, 1.0, 1.0, 1.0)
}

pub fn edge_color() -> Vec4 {
    vec4(0.0, 0.0, 0.0, 1.0)
}

pub const EDGE_WIDTH: f32 = 1.0;

pub fn token_color(token: Token) -> Vec4 {
    match token {
        Token::Plain => ink_color(),
        Token::Keyword => vec4(0.80, 0.70, 1.0, 1.0),
        Token::Type => vec4(0.58, 0.88, 0.94, 1.0),
        Token::Text => vec4(0.76, 0.92, 0.64, 1.0),
        Token::Number | Token::Attribute => vec4(0.98, 0.80, 0.54, 1.0),
        Token::Comment => vec4(1.0, 1.0, 1.0, 0.55),
    }
}

pub trait BookInk {
    fn ink(self) -> Self;
    fn ink_dim(self) -> Self;
    fn ink_colored(self, color: Vec4) -> Self;
}

impl BookInk for UiNodeBuilder<'_, '_, '_> {
    fn ink(self) -> Self {
        self.color_raw::<UiBase>(ink_color())
            .with_text_outline(edge_color(), EDGE_WIDTH)
    }

    fn ink_dim(self) -> Self {
        self.color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 0.72))
            .with_text_outline(edge_color(), EDGE_WIDTH)
    }

    fn ink_colored(self, color: Vec4) -> Self {
        self.color_raw::<UiBase>(color)
            .with_text_outline(edge_color(), EDGE_WIDTH)
    }
}
