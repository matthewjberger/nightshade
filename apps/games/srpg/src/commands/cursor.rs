use crate::components::CursorPart;
use crate::data::CursorPartKind;
use crate::resources::Sheet;
use crate::tags::Upright;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn spawn_flat(commands: &mut Commands, sprite: Sprite, position: Vec3) -> Entity {
    let transform = LocalTransform {
        translation: position,
        rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    commands.spawn((
        transform,
        GlobalTransform(transform.as_matrix()),
        Visibility { visible: true },
        sprite,
    ))
}

pub fn spawn_cursor(commands: &mut Commands, sheet: &Sheet) {
    let flat = nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());
    let band = Sprite::new(theme::ATLAS, sheet.cell)
        .with_size(theme::RING_SIZE.x, theme::RING_SIZE.y)
        .with_color(theme::RING_PLAYER)
        .with_layer(theme::LAYER_GROUND);
    let ring = spawn_sprite(commands, band, Vec3::zeros());
    commands.set(
        ring,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: flat,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(
        ring,
        CursorPart {
            kind: CursorPartKind::Ring,
        },
    );
    commands.set(ring, Visibility { visible: false });

    let bracket = Sprite::new(
        theme::ATLAS,
        crate::queries::sheet::tile(sheet, theme::CURSOR_BRACKET),
    )
    .with_size(theme::CURSOR_SIZE, theme::CURSOR_SIZE)
    .with_color(theme::CURSOR_COLOR)
    .with_layer(theme::LAYER_GROUND);
    let marker = spawn_sprite(commands, bracket, Vec3::zeros());
    commands.set(
        marker,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: flat,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(
        marker,
        CursorPart {
            kind: CursorPartKind::Bracket,
        },
    );
    commands.set(marker, Visibility { visible: false });

    let (region, height) = crate::queries::sheet::standing(sheet, theme::POINTER);
    let pointer = Sprite::new(theme::ATLAS, region)
        .with_size(theme::ARROW_SIZE, theme::ARROW_SIZE * height)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_color(theme::ARROW_COLOR)
        .with_layer(theme::LAYER_STANDING);
    let arrow = spawn_sprite(commands, pointer, Vec3::zeros());
    commands.add_tag_type::<Upright>(arrow);
    commands.set(
        arrow,
        CursorPart {
            kind: CursorPartKind::Arrow,
        },
    );
    commands.set(arrow, Visibility { visible: false });
}
