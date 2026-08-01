use crate::commands::effects::sparkle;
use crate::commands::pocket::{add_item, show_toast};
use crate::components::Upright;
use crate::data::Item;
use crate::resources::{Fishing, FishingState, Inventory, Sheet, Toast};
use crate::tags::Bobber;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_sprite::prelude::*;

pub fn cast(
    commands: &mut Commands,
    sheet: &Sheet,
    fishing: &mut Fishing,
    toast: &mut Toast,
    water: Vec2,
) {
    let wait =
        theme::FISH_WAIT_MIN + fishing.rng.f32() * (theme::FISH_WAIT_MAX - theme::FISH_WAIT_MIN);
    fishing.state = FishingState::Waiting(wait);
    fishing.spot = water;
    let sprite = Sprite::new(theme::ATLAS, sheet.shadow)
        .with_size(0.22, 0.22)
        .with_color([0.95, 0.25, 0.2, 1.0])
        .with_layer(theme::LAYER_STANDING);
    let bobber = spawn_sprite(
        commands,
        sprite,
        Vec3::new(water.x, theme::WATER_LEVEL + 0.02, water.y),
    );
    commands.add_tag_type::<Bobber>(bobber);
    commands.set(bobber, Upright::default());
    fishing.bobber = Some(bobber);
    show_toast(toast, "Cast. Wait for the tug.");
}

pub fn reel_in(commands: &mut Commands, fishing: &mut Fishing) {
    if let Some(bobber) = fishing.bobber.take() {
        commands.despawn(bobber);
    }
    fishing.state = FishingState::Idle;
}

pub fn land(
    commands: &mut Commands,
    fishing: &mut Fishing,
    inventory: &mut Inventory,
    toast: &mut Toast,
) {
    let splash = Vec3::new(fishing.spot.x, theme::WATER_LEVEL + 0.2, fishing.spot.y);
    reel_in(commands, fishing);
    sparkle(commands, splash);
    add_item(inventory, Item::Fish, 1);
    show_toast(toast, "A fish!");
}
