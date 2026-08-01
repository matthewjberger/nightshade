use crate::commands::life::spawn_standing;
use crate::components::{Count, Held, Hop, Spot};
use crate::data::{self, Item, Placement};
use crate::queries::cylinder;
use crate::queries::sheet::item_picture;
use crate::resources::Sheet;
use fastrand::Rng;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;

pub fn drop_item(
    commands: &mut Commands,
    sheet: &Sheet,
    turntable: Entity,
    item: Item,
    spot: Vec2,
    lift: f32,
) -> Entity {
    let placement = Placement::new(spot, lift)
        .with_scale(data::PICKUP_SCALE)
        .with_shadow(Vec2::new(0.4, 0.24));
    let entity = spawn_standing(
        commands,
        sheet,
        turntable,
        item_picture(sheet, item),
        placement,
    );
    commands.set(entity, Held(item));
    commands.set(entity, Count(1));
    commands.set(entity, Spot(spot));
    commands.set(entity, Hop(1.0));
    entity
}

pub fn scatter_items(
    commands: &mut Commands,
    sheet: &Sheet,
    turntable: Entity,
    item: Item,
    around: Vec2,
    lift: f32,
    count: u32,
    rng: &mut Rng,
) {
    for _ in 0..count {
        let turn = rng.f32() * std::f32::consts::TAU;
        let reach = 0.4 + 0.5 * rng.f32();
        let spot = Vec2::new(
            around.x + turn.cos() * reach,
            cylinder::wrap(around.y + turn.sin() * reach),
        );
        drop_item(commands, sheet, turntable, item, spot, lift);
    }
}
