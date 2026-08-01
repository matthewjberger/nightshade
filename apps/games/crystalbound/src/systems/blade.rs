use crate::commands::{roll_below, show_toast};
use crate::components::{Facing, Health, Hurt, Life, Spent, SwingRest};
use crate::contract::MonsterKind;
use crate::data;
use crate::data::Ask;
use crate::resources::{Party, Rig, Toast};
use crate::tags::FieldPiece;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Swingers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item Facing, &'item mut SwingRest, &'item LocalTransform),
>;
pub type Blades<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item mut Life, &'item mut Spent, &'item LocalTransform),
>;
pub type Prey<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item MonsterKind,
        &'item mut Health,
        &'item mut Hurt,
        &'item mut Sprite,
        &'item LocalTransform,
    ),
>;

pub fn swing_the_blade(
    actions: Res<Actions<Ask>>,
    rig: Res<Rig>,
    mut commands: Commands,
    mut swingers: Swingers<'_, '_>,
) {
    if !actions.just_pressed(Ask::Swing) {
        return;
    }
    let sheet = rig.sheets.dungeon_tile(theme::BLADE_TILE);
    let mut swings = Vec::new();
    swingers.for_each_mut(|_entity, (facing, swing_rest, transform)| {
        if swing_rest.0 > 0.0 {
            return;
        }
        swing_rest.0 = theme::SWING_REST;
        let here = Vec2::new(transform.translation.x, transform.translation.y);
        swings.push((here + facing.0 * theme::BLADE_REACH, facing.0));
    });

    for (spot, facing) in swings {
        let sprite = Sprite::new(theme::ATLAS, sheet)
            .with_size(0.9, 0.9)
            .with_layer(theme::LAYER_BLADE)
            .with_flip(facing.x < 0.0, facing.y < 0.0);
        let entity = spawn_sprite(&mut commands, sprite, Vec3::new(spot.x, spot.y, 0.0));
        commands.set(entity, Life(theme::BLADE_LIFE));
        commands.set(entity, Spent(false));
        commands.add_tag_type::<FieldPiece>(entity);
    }
}

pub fn land_the_blows(
    time: Res<Time>,
    mut rig: ResMut<Rig>,
    mut party: ResMut<Party>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    mut blades: Blades<'_, '_>,
    mut prey: Prey<'_, '_>,
) {
    let delta_time = time.delta_time;
    let mut strikes = Vec::new();
    let mut spent = Vec::new();
    blades.for_each_mut(|entity, (life, already_spent, transform)| {
        life.0 -= delta_time;
        if life.0 <= 0.0 {
            spent.push(entity);
            return;
        }
        if already_spent.0 {
            return;
        }
        already_spent.0 = true;
        strikes.push(Vec2::new(transform.translation.x, transform.translation.y));
    });
    for entity in spent {
        commands.despawn_recursive(entity);
    }
    if strikes.is_empty() {
        return;
    }

    let power =
        party.heroes.first().map(|hero| hero.power()).unwrap_or(6) + party.blade_level as i32 * 3;

    let mut felled = Vec::new();
    let mut winnings = 0;
    let mut hits = 0;
    prey.for_each_mut(|entity, (kind, health, hurt, sprite, transform)| {
        let here = Vec2::new(transform.translation.x, transform.translation.y);
        if !strikes
            .iter()
            .any(|strike| (here - strike).magnitude() < theme::BLADE_SPAN)
        {
            return;
        }
        let stats = data::monster_stats(*kind);
        health.0 -= (power - stats.guard).max(1);
        hurt.0 = 0.22;
        sprite.color = theme::HURT_TINT;
        hits += 1;
        if health.0 <= 0 {
            winnings += stats.gold;
            felled.push((entity, kind.name()));
        }
    });

    if hits > 0 {
        rig.shake = rig.shake.max(0.14);
    }
    let mut last = None;
    for (entity, name) in felled {
        commands.despawn_recursive(entity);
        last = Some(name);
    }
    if winnings > 0 {
        party.gold += winnings;
        let extra = roll_below(&mut rig, 3);
        party.gold += extra;
        if let Some(name) = last {
            show_toast(
                &mut toast,
                format!("The {name} falls. {} gold.", winnings + extra),
            );
        }
    }
}
