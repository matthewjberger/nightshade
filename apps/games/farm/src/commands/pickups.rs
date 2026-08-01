use crate::components::{Count, Lit, Upright};
use crate::data::Item;
use crate::queries::sheet::standing;
use crate::resources::Sheet;
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_sprite::prelude::*;

pub fn drop_pickups(
    commands: &mut Commands,
    sheet: &Sheet,
    position: Vec3,
    item: Item,
    tile: u32,
    count: u32,
) {
    let mut rng = Rng::with_seed(
        ((position.x.abs() * 71.0) as u32 + (position.z.abs() * 149.0) as u32 + count + 7) as u64,
    );
    for _ in 0..count {
        let turn = rng.f32() * std::f32::consts::TAU;
        let reach = theme::PICKUP_SPREAD * (0.35 + 0.65 * rng.f32());
        let spot = position + Vec3::new(turn.cos() * reach, 0.0, turn.sin() * reach);
        let (region, height) = standing(sheet, tile);
        let sprite = Sprite::new(theme::ATLAS, region)
            .with_size(theme::PICKUP_SIZE, height * theme::PICKUP_SIZE)
            .with_anchor(Vec2::new(0.5, 0.0))
            .with_layer(theme::LAYER_STANDING);
        let entity = spawn_sprite(commands, sprite, spot);
        commands.set(entity, Upright::default());
        commands.set(
            entity,
            Lit {
                base: [1.0, 1.0, 1.0, 1.0],
            },
        );
        commands.set(entity, item);
        commands.set(entity, Count(1));
    }
}
