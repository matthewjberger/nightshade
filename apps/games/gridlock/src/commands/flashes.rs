use crate::components::Fade;
use crate::data::look::{FLASH_COUNT, FLASH_OUTLINE, FLASH_SINK};
use crate::queries::score::{flash_color, flash_pop, flash_text, flash_value};
use crate::resources::{Flash, Flashes};
use nightshade::prelude::*;
use nightshade_text::prelude::*;

pub fn spawn_flash_pool(
    text_state: &mut TextState,
    commands: &mut Commands,
    size: f32,
) -> Vec<Entity> {
    (0..FLASH_COUNT)
        .map(|_| {
            let entity = spawn_3d_billboard_text_with_properties(
                text_state,
                commands,
                "",
                Vec3::new(0.0, FLASH_SINK, 0.0),
                TextProperties {
                    font_size: size,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    alignment: TextAlignment::Center,
                    vertical_alignment: VerticalAlignment::Middle,
                    outline_width: FLASH_OUTLINE,
                    outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
            commands.set(entity, Fade(0.0));
            entity
        })
        .collect()
}

pub fn score_hit(flashes: &mut Flashes, position: Vec3, base: u64, combo_seconds: f32) {
    flashes.streak += 1;
    flashes.streak_left = combo_seconds;
    let streak = flashes.streak;
    let value = flash_value(base, streak);
    flashes.pending.push(Flash {
        position,
        text: flash_text(value, streak),
        color: flash_color(streak),
        pop: flash_pop(streak),
    });
}
