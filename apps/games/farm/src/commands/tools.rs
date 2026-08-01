use crate::commands::effects::{leaves, sparkle};
use crate::commands::life::spawn_stump;
use crate::commands::pickups::drop_pickups;
use crate::commands::pocket::{add_item, show_toast};
use crate::components::{Bush, Flip, Hits, Remaining, Shake, Wood};
use crate::data::{Item, Tool};
use crate::queries::sheet::{standing, tile};
use crate::queries::tools::swing_angle;
use crate::resources::{Inventory, Sheet, Toast};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3, Vec4};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn swing(commands: &mut Commands, sheet: &Sheet, farmer: Entity, tool: Tool, flip: bool) {
    swing_region(commands, farmer, tile(sheet, tool.icon()), flip);
}

pub fn swing_region(commands: &mut Commands, farmer: Entity, region: SpriteRegion, flip: bool) {
    let side = if flip { -1.0 } else { 1.0 };
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(theme::SWING_TOOL_SIZE, theme::SWING_TOOL_SIZE)
        .with_anchor(Vec2::new(0.5, 0.15))
        .with_flip(!flip, false)
        .with_layer(theme::LAYER_STANDING);
    let held = spawn_sprite(commands, sprite, Vec3::new(0.0, 0.0, 0.0));
    commands.set(
        held,
        LocalTransform {
            translation: Vec3::new(
                theme::SWING_OFFSET.x * side,
                theme::SWING_OFFSET.y,
                theme::SWING_OFFSET.z,
            ),
            rotation: nalgebra_glm::quat_angle_axis(swing_angle(0.0, flip), &Vec3::z()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(held, ChildOf(farmer));
    commands.set(
        held,
        crate::components::Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    commands.set(held, Remaining(theme::SWING_SECONDS));
    commands.set(held, Flip(flip));
}

pub fn chop(
    commands: &mut Commands,
    sheet: &Sheet,
    toast: &mut Toast,
    entity: Entity,
    hits: &mut Hits,
    wood: &Wood,
    position: Vec3,
    day: u32,
    power: u32,
) {
    hits.0 = hits.0.saturating_sub(power.max(1));
    leaves(
        commands,
        position + Vec3::new(0.0, 1.2, 0.0),
        Vec4::new(0.45, 0.7, 0.35, 1.0),
    );
    if hits.0 > 0 {
        commands.set(
            entity,
            Shake {
                remaining: theme::SHAKE_SECONDS,
            },
        );
        return;
    }
    commands.despawn_recursive(entity);
    spawn_stump(commands, sheet, position, day);
    sparkle(commands, position + Vec3::new(0.0, 0.5, 0.0));
    drop_pickups(
        commands,
        sheet,
        position,
        Item::Wood,
        theme::LOG_TILE,
        wood.0,
    );
    show_toast(toast, "Felled the tree");
}

pub fn clear_stump(
    commands: &mut Commands,
    sheet: &Sheet,
    toast: &mut Toast,
    entity: Entity,
    position: Vec3,
) {
    commands.despawn_recursive(entity);
    sparkle(commands, position + Vec3::new(0.0, 0.3, 0.0));
    drop_pickups(
        commands,
        sheet,
        position,
        Item::Wood,
        theme::LOG_TILE,
        theme::STUMP_WOOD,
    );
    show_toast(toast, "Cleared the stump");
}

pub fn dig(
    commands: &mut Commands,
    sheet: &Sheet,
    toast: &mut Toast,
    entity: Entity,
    position: Vec3,
) {
    commands.despawn_recursive(entity);
    sparkle(commands, position + Vec3::new(0.0, 0.3, 0.0));
    drop_pickups(
        commands,
        sheet,
        position,
        Item::Stone,
        theme::ROCK_PILE,
        theme::STONE_PER_ROCKS,
    );
    show_toast(toast, "Dug up the rocks");
}

pub fn pick(
    commands: &mut Commands,
    sheet: &Sheet,
    inventory: &mut Inventory,
    toast: &mut Toast,
    bush: &mut Bush,
    sprite: &mut Sprite,
    position: Vec3,
    day: u32,
) {
    bush.picked_day = Some(day);
    let (region, height) = standing(sheet, theme::BUSH);
    sprite.region = region;
    sprite.size.y = height;
    sparkle(commands, position + Vec3::new(0.0, 0.5, 0.0));
    add_item(inventory, Item::Berry, theme::BERRIES_PER_BUSH);
    show_toast(toast, format!("Picked {} berries", theme::BERRIES_PER_BUSH));
}

pub fn milk(commands: &mut Commands, inventory: &mut Inventory, toast: &mut Toast, position: Vec3) {
    sparkle(commands, position + Vec3::new(0.0, 0.5, 0.0));
    add_item(inventory, Item::Milk, 1);
    show_toast(toast, "A pail of milk");
}
