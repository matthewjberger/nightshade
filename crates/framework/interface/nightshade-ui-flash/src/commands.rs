use crate::data::Wash;
use crate::resources::{Flash, FlashHandles};
use nalgebra_glm::Vec4;
use nightshade::prelude::vec2;
use nightshade_ui::prelude::*;

pub fn add_flash(tree: &mut UiTreeBuilder) -> FlashHandles {
    let panel = tree
        .add_node()
        .window(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
        .without_pointer_events()
        .entity();
    FlashHandles { panel }
}

pub fn flash(flash: &mut Flash, color: Vec4, fade: f32) {
    flash.washes.push(Wash {
        color,
        amount: 1.0,
        fade,
    });
}

pub fn clear(flash: &mut Flash) {
    flash.washes.clear();
}
