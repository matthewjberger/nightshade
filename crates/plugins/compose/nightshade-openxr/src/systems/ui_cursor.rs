use crate::contract::XrUiPointer;
use nightshade::prelude::*;
use nightshade_paint::data::DrawRect;

const RING_SIZE: f32 = 18.0;
const RING_BORDER: f32 = 2.0;
const PRESSED_SIZE: f32 = 12.0;
const DOT_SIZE: f32 = 4.0;
const CURSOR_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.95);
const NO_FILL: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
const CURSOR_LAYER: u8 = u8::MAX;

pub fn ui_cursor_system(
    pointer: Res<XrUiPointer>,
    mut paint: ResMut<nightshade_paint::PaintFrame>,
) {
    let Some(hit) = pointer.hit else {
        return;
    };
    if pointer.pressed {
        push_disc(&mut paint, hit, PRESSED_SIZE, CURSOR_COLOR, 0.0, 1);
    } else {
        push_disc(&mut paint, hit, RING_SIZE, NO_FILL, RING_BORDER, 0);
        push_disc(&mut paint, hit, DOT_SIZE, CURSOR_COLOR, 0.0, 1);
    }
}

fn push_disc(
    paint: &mut nightshade_paint::PaintFrame,
    center: Vec2,
    size: f32,
    color: Vec4,
    border_width: f32,
    z_index: i32,
) {
    paint.rects.push(DrawRect {
        position: center - Vec2::new(size * 0.5, size * 0.5),
        size: Vec2::new(size, size),
        color,
        corner_radius: size * 0.5,
        border_width,
        border_color: CURSOR_COLOR,
        layer: CURSOR_LAYER,
        z_index,
        ..Default::default()
    });
    paint.rect_entities.push(None);
}
