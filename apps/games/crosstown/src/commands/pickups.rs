use crate::components::{
    Breaks, Facing, Flash, Flying, Heals, Health, Kind, Lift, Piece, Rise, Solid, Weapon,
};
use crate::data;
use crate::data::Look;
use crate::queries::{break_spots, noise, pickup_spots};
use crate::resources::Sheets;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_with_material;
use nightshade_scene::prelude::Lifetime;
use nightshade_sprite::prelude::{Sprite, spawn_sprite};

pub fn scatter_breakables(commands: &mut Commands, look: Look, end: f32, stage: u32) {
    for (kind, at) in break_spots(look, end, stage) {
        spawn_breakable(commands, kind, at, stage);
    }
}

pub fn scatter_pickups(commands: &mut Commands, sheets: &Sheets, end: f32, stage: u32) {
    for (kind, at) in pickup_spots(end, stage) {
        spawn_pickup(commands, sheets, kind, at, stage);
    }
}

pub fn spawn_breakable(commands: &mut Commands, kind: u32, at: Vec3, stage: u32) -> Entity {
    let plan = &data::BREAK_KINDS[kind as usize];
    let entity = spawn_mesh_with_material(
        commands,
        "Cube",
        at + Vec3::new(0.0, plan.size[1] * 0.5, 0.0),
        Vec3::new(plan.size[0], plan.size[1], plan.size[2]),
        plan.material,
    );
    commands.set(entity, Health(plan.health));
    commands.set(entity, Breaks(kind));
    commands.set(entity, Solid(plan.radius));
    commands.set(entity, Flash(0.0));
    commands.set(entity, Piece(stage));
    entity
}

pub fn burst(commands: &mut Commands, kind: u32, at: Vec3, stage: u32) {
    let plan = &data::BREAK_KINDS[kind as usize];
    for shard in 0..plan.shards {
        let spin = noise(shard * 37 + stage * 11) * std::f32::consts::TAU;
        let lift = 0.2 + noise(shard * 53 + stage * 7) * plan.size[1];
        let entity = spawn_mesh_with_material(
            commands,
            "Cube",
            at + Vec3::new(spin.cos() * 0.2, lift, spin.sin() * 0.2),
            Vec3::new(data::SHARD_SIZE, data::SHARD_SIZE, data::SHARD_SIZE),
            plan.material,
        );
        commands.set(entity, Lift(lift));
        commands.set(
            entity,
            Rise(data::SHARD_RISE * (0.5 + noise(shard * 29 + stage) * 0.8)),
        );
        commands.set(
            entity,
            crate::components::Fling((noise(shard * 71 + stage) - 0.5) * data::SHARD_SPREAD * 2.0),
        );
        commands.set(
            entity,
            Lifetime {
                remaining: data::SHARD_LIFE,
            },
        );
        commands.set(entity, Piece(stage));
    }
}

pub fn spawn_weapon(
    commands: &mut Commands,
    sheets: &Sheets,
    kind: u32,
    at: Vec3,
    stage: u32,
) -> Entity {
    let plan = &data::WEAPON_KINDS[kind as usize];
    let region = sheets
        .weapons
        .get(kind as usize)
        .copied()
        .unwrap_or_default();
    let sprite = Sprite::new(data::ATLAS, region)
        .with_size(plan.size[0], plan.size[1])
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_color(plan.color)
        .with_layer(data::LAYER_FIGHTER);
    let entity = spawn_sprite(commands, sprite, at);
    commands.set(entity, Weapon(kind));
    commands.set(entity, Piece(stage));
    entity
}

pub fn spawn_pickup(
    commands: &mut Commands,
    sheets: &Sheets,
    kind: u32,
    at: Vec3,
    stage: u32,
) -> Entity {
    let plan = &data::PICKUP_KINDS[kind as usize];
    let region = sheets
        .pickups
        .get(kind as usize)
        .copied()
        .unwrap_or_default();
    let sprite = Sprite::new(data::ATLAS, region)
        .with_size(plan.size[0], plan.size[1])
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_color(plan.color)
        .with_layer(data::LAYER_FIGHTER);
    let entity = spawn_sprite(commands, sprite, at);
    commands.set(entity, Heals(plan.heal));
    commands.set(entity, Kind(kind));
    commands.set(entity, Piece(stage));
    entity
}

pub fn drop_from(commands: &mut Commands, sheets: &Sheets, step: u32, at: Vec3, stage: u32) {
    match data::DROP_TABLE[step as usize % data::DROP_TABLE.len()] {
        data::DROP_ROAST => {
            spawn_pickup(commands, sheets, data::ROAST, at, stage);
        }
        data::DROP_DRINK => {
            spawn_pickup(commands, sheets, data::DRINK, at, stage);
        }
        data::DROP_PIPE => {
            spawn_weapon(commands, sheets, data::PIPE, at, stage);
        }
        data::DROP_BOTTLE => {
            spawn_weapon(commands, sheets, data::BOTTLE, at, stage);
        }
        _ => {}
    }
}

pub fn throw_weapon(
    commands: &mut Commands,
    sheets: &Sheets,
    kind: u32,
    at: Vec3,
    facing: f32,
    stage: u32,
) {
    let entity = spawn_weapon(
        commands,
        sheets,
        kind,
        at + Vec3::y() * data::THROW_LIFT,
        stage,
    );
    commands.set(
        entity,
        Flying(facing * data::WEAPON_KINDS[kind as usize].thrown),
    );
    commands.set(entity, Facing(facing));
    commands.set(entity, Lift(data::THROW_LIFT));
    commands.set(entity, Rise(0.0));
}
