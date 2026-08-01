use crate::components::{Block, Progress, Stage, Watered};
use crate::theme;
use blocker_blocks::data::crops::CropKind;
use blocker_blocks::queries::{liquid_near, sheet_standing};
use blocker_blocks::resources::{Sheet, VoxelWorld};
use nalgebra_glm::{IVec3, Vec2, Vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_sprite::prelude::*;

pub fn plant(
    commands: &mut Commands,
    world: &VoxelWorld,
    sheet: &Sheet,
    kind: CropKind,
    block: IVec3,
) -> Entity {
    let (region, height) = sheet_standing(sheet, kind.info().stages[0]);
    let sprite = Sprite::new(blocker_blocks::data::atlas::SPRITE_ATLAS, region)
        .with_size(0.9, height * 0.9)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::SPRITE_LAYER)
        .as_billboard();
    let position = Vec3::new(
        block.x as f32 + 0.5,
        block.y as f32 + 1.0,
        block.z as f32 + 0.5,
    );
    let entity = spawn_sprite(commands, sprite, position);
    commands.set(entity, kind);
    commands.set(entity, Block(block));
    commands.set(entity, Progress(0.0));
    commands.set(entity, Stage(0));
    commands.set(
        entity,
        Watered(liquid_near(
            world,
            block,
            theme::WATER_REACH,
            blocker_blocks::data::blocks::WATER,
        )),
    );
    entity
}
