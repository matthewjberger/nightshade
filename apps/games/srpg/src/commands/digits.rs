use crate::components::{Age, Tint};
use crate::queries::digit_offset;
use crate::resources::Sheet;
use crate::tags::Upright;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn spawn_digits(
    commands: &mut Commands,
    sheet: &Sheet,
    position: Vec3,
    text: &str,
    color: [f32; 4],
) {
    let count = text.chars().count();
    let holder = commands.spawn((
        LocalTransform {
            translation: position,
            ..Default::default()
        },
        GlobalTransform::default(),
    ));
    commands.add_tag_type::<Upright>(holder);
    commands.set(holder, Age(0.0));
    commands.set(holder, Tint(color));
    for (slot, glyph) in text.chars().enumerate() {
        let tile = match glyph.to_digit(10) {
            Some(digit) => theme::DIGIT_TILE + digit,
            None => theme::QUESTION_TILE,
        };
        let sprite = Sprite::new(theme::ATLAS, crate::queries::sheet::tile(sheet, tile))
            .with_size(theme::DIGIT_SIZE, theme::DIGIT_SIZE)
            .with_anchor(Vec2::new(0.5, 0.0))
            .with_color(color)
            .with_layer(theme::LAYER_STANDING + 1);
        let entity = spawn_sprite(commands, sprite, position);
        commands.set(
            entity,
            LocalTransform {
                translation: digit_offset(slot, count),
                ..Default::default()
            },
        );
        commands.set(entity, nightshade::ecs::ChildOf(holder));
    }
}
