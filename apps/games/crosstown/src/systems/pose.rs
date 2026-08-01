use crate::components::{Book, Facing, Flash, Lift, Pose, Tint};
use crate::data;
use crate::resources::Sheets;
use nightshade::prelude::*;
use nightshade_sprite::prelude::Sprite;

pub fn dress(sheets: Res<Sheets>, fighters: Fighters) {
    let pages = &sheets.pages;
    fighters.for_each(|_entity, (book, pose, facing, lift, flash, tint, sprite)| {
        let Some(page) = pages.get(book.0 as usize) else {
            return;
        };
        let region = page.cell(data::PAGE_COLUMNS, data::PAGE_ROWS, pose.0);
        let flip = facing.0 < 0.0;
        let anchor = -lift.0 / data::FIGHTER_HEIGHT;
        if sprite.region != region {
            sprite.region = region;
        }
        if sprite.flip_x != flip {
            sprite.flip_x = flip;
        }
        if sprite.anchor.y != anchor {
            sprite.anchor.y = anchor;
        }
        let gain = 1.0 + data::FLASH_GAIN * (flash.0 / data::FLASH_TIME).clamp(0.0, 1.0);
        let color = [
            tint.0[0] * gain,
            tint.0[1] * gain,
            tint.0[2] * gain,
            tint.0[3],
        ];
        if sprite.color != color {
            sprite.color = color;
        }
    });
}

pub type Fighters<'world, 'item> = Query<
    'world,
    (
        &'item Book,
        &'item Pose,
        &'item Facing,
        &'item Lift,
        &'item Flash,
        &'item Tint,
        &'item mut Sprite,
    ),
>;
