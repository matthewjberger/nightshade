use crate::components::{Air, Fall, Food, Grounded, Motion, Riding, Spawn, Submerged, Velocity};
use crate::data::settings::Settings;
use crate::theme;
use blocker_blocks::queries::block_at;
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_screen::data::notice::Notice;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_health::{Health, PendingDamage, commands::heal, commands::hurt, queries::fallen};
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

type Moving<'world, 'item> = Query<
    'world,
    (
        &'item mut LocalTransform,
        &'item mut Velocity,
        &'item Motion,
        &'item Grounded,
        &'item Submerged,
        Option<&'item Riding>,
    ),
>;

pub fn keep_the_body(
    world: Res<VoxelWorld>,
    time: Res<Time>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut notices: EventWriter<Notice>,
    mut commands: Commands,
    mut vitals: Vitals<'_, '_>,
    mut moving: Moving<'_, '_>,
) {
    let delta_time = time.delta_time.min(0.05);
    let Some(body) = hero.0 else {
        return;
    };
    let Some((position, motion, grounded, submerged, riding)) = read_the_hero(
        &hero,
        &mut moving,
        |(transform, _, motion, grounded, submerged, riding)| {
            (
                transform.translation,
                *motion,
                grounded.0,
                submerged.0,
                riding.is_some(),
            )
        },
    ) else {
        return;
    };

    let Some((mut health, mut food, mut air, mut fall, home)) =
        read_the_hero(&hero, &mut vitals, |(health, _, food, air, fall, spawn)| {
            (health.0, food.0, air.0, fall.0, spawn.0)
        })
    else {
        return;
    };
    let mut hurt_by = 0.0;
    let drown_seconds = config.number(Settings::DrownSeconds);

    if motion == Motion::Watch {
        health = crate::data::body::FULL_HEALTH;
        food = crate::data::body::FULL_FOOD;
        air = 0.0;
        fall = position.y;
        write_the_vitals(&mut vitals, body, health, food, air, fall, 0.0);
        return;
    }

    let food_burn = config.number(Settings::FoodBurn);
    food = (food - food_burn * delta_time).max(0.0);
    if food >= config.number(Settings::FoodHeal) && health < crate::data::body::FULL_HEALTH {
        heal(
            &mut health,
            crate::data::body::FULL_HEALTH,
            config.number(Settings::HealRate) * delta_time,
        );
        food = (food - food_burn * delta_time * 3.0).max(0.0);
    } else if food <= 0.0 {
        hurt_by += config.number(Settings::StarveRate) * delta_time;
    }

    let head = position + Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
    let block = IVec3::new(
        head.x.floor() as i32,
        head.y.floor() as i32,
        head.z.floor() as i32,
    );
    if world
        .blocks
        .fill(block_at(&world, block).index())
        .is_some_and(|fill| fill.liquid == blocker_blocks::data::blocks::WATER)
    {
        air += delta_time;
        if air >= drown_seconds {
            hurt_by += config.number(Settings::DrownRate) * delta_time;
        }
    } else {
        air = 0.0;
    }

    if motion == Motion::Fly || riding || submerged {
        fall = position.y;
    } else if grounded {
        let drop = fall - position.y;
        let fall_floor = config.number(Settings::FallFloor);
        if drop > fall_floor {
            hurt_by += (drop - fall_floor) * config.number(Settings::FallHurt);
        }
        fall = position.y;
    } else if position.y > fall {
        fall = position.y;
    }

    if fallen(&Health(health)) {
        health = crate::data::body::FULL_HEALTH;
        hurt_by = 0.0;
        food = crate::data::body::FULL_FOOD * 0.6;
        air = 0.0;
        fall = home.y;
        moving.for_entity(body, |(transform, velocity, _, _, _, _)| {
            transform.translation = home;
            velocity.0 = Vec3::zeros();
        });
        if riding {
            commands.remove::<(Riding,)>(body);
        }
        notices.send(Notice("You woke up back at the village".into()));
    }

    write_the_vitals(&mut vitals, body, health, food, air, fall, hurt_by);
}

type Vitals<'world, 'item> = Query<
    'world,
    (
        &'item mut Health,
        &'item mut PendingDamage,
        &'item mut Food,
        &'item mut Air,
        &'item mut Fall,
        &'item Spawn,
    ),
>;

fn write_the_vitals(
    vitals: &mut Vitals<'_, '_>,
    body: Entity,
    health: f32,
    food: f32,
    air: f32,
    fall: f32,
    hurt_by: f32,
) {
    vitals.for_entity(body, |(kept, pending, fed, breathed, dropped, _)| {
        kept.0 = health;
        hurt(pending, hurt_by);
        fed.0 = food;
        breathed.0 = air;
        dropped.0 = fall;
    });
}
