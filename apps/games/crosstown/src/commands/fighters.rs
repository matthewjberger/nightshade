use crate::components::{
    Ammo, Book, Chief, Combo, Dash, Facing, Flash, Fling, Health, Holding, Kind, Leap, Lift, Piece,
    Pose, Rest, Rise, Slot, Solid, Speed, Step, Stride, Strike, Stun, Tap, Tint, Toward, Wish,
};
use crate::data;
use crate::resources::Sheets;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn spawn_hero(commands: &mut Commands, sheets: &Sheets, at: Vec3) -> Entity {
    let hero = spawn_body(
        commands,
        sheets,
        data::HERO_PAGE,
        at,
        [1.0, 1.0, 1.0, 1.0],
        1.0,
        u32::MAX,
    );
    commands.set(hero, Health(data::HERO_HEALTH));
    commands.set(hero, Speed(data::HERO_SPEED));
    commands.set(hero, Combo(0.0));
    commands.set(hero, Holding(0));
    commands.set(hero, Ammo(0));
    commands.set(hero, Wish(0.0));
    commands.set(hero, Leap(0.0));
    commands.set(hero, Tap(0.0));
    commands.set(hero, Toward(0.0));
    commands.set(hero, Dash(0.0));
    commands.set(hero, Solid(data::HERO_RADIUS));
    hero
}

pub fn spawn_thug(
    commands: &mut Commands,
    sheets: &Sheets,
    kind: u32,
    at: Vec3,
    facing: f32,
    slot: f32,
    stage: u32,
) -> Entity {
    let plan = &data::THUG_KINDS[kind as usize];
    let thug = spawn_body(
        commands, sheets, plan.page, at, plan.tint, plan.scale, stage,
    );
    commands.set(thug, Health(plan.health));
    commands.set(thug, Speed(plan.speed));
    commands.set(thug, Kind(kind));
    commands.set(thug, Slot(slot));
    commands.set(thug, Rest(plan.rest));
    commands.set(thug, Facing(facing));
    commands.set(thug, Solid(plan.radius));
    if plan.chief {
        commands.set(thug, Chief(kind));
    }
    thug
}

fn spawn_body(
    commands: &mut Commands,
    sheets: &Sheets,
    page: u32,
    at: Vec3,
    color: [f32; 4],
    scale: f32,
    stage: u32,
) -> Entity {
    let region = sheets.pages[page as usize].cell(data::PAGE_COLUMNS, data::PAGE_ROWS, 0);
    let sprite = Sprite::new(data::ATLAS, region)
        .with_size(data::FIGHTER_WIDTH * scale, data::FIGHTER_HEIGHT * scale)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_color(color)
        .with_layer(data::LAYER_FIGHTER);
    let body = spawn_sprite(commands, sprite, at);
    commands.set(body, Strike(0.0));
    commands.set(body, Stun(0.0));
    commands.set(body, Stride(0.0));
    commands.set(body, Lift(0.0));
    commands.set(body, Rise(0.0));
    commands.set(body, Fling(0.0));
    commands.set(body, Book(page));
    commands.set(body, Pose(data::POSE_IDLE));
    commands.set(body, Step(0));
    commands.set(body, Facing(1.0));
    commands.set(body, Flash(0.0));
    commands.set(body, Tint(color));
    commands.set(body, Piece(stage));
    spawn_blob(commands, sheets, body, scale);
    body
}

fn spawn_blob(commands: &mut Commands, sheets: &Sheets, owner: Entity, scale: f32) {
    let sprite = Sprite::new(data::ATLAS, sheets.shadow)
        .with_size(data::SHADOW_SIZE[0] * scale, data::SHADOW_SIZE[1] * scale)
        .with_color([0.0, 0.0, 0.0, data::SHADOW_ALPHA])
        .with_layer(data::LAYER_SHADOW);
    let blob = spawn_sprite(commands, sprite, Vec3::new(0.0, 0.0, 0.0));
    commands.set(
        blob,
        LocalTransform {
            translation: Vec3::new(
                0.0,
                data::SHADOW_LIFT,
                data::SHADOW_SIZE[1] * data::SHADOW_FORWARD * scale,
            ),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(blob, ChildOf(owner));
}
