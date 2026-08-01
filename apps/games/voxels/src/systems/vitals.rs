use crate::commands::toast_show;
use crate::queries::block_at;
use crate::resources::{Player, Toast, VoxelWorld};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;

pub fn keep_the_body(
    mut player: ResMut<Player>,
    world: Res<VoxelWorld>,
    time: Res<Time>,
    mut toast: ResMut<Toast>,
) {
    let delta_time = time.delta_time.min(0.05);
    player.hurt = (player.hurt - delta_time).max(0.0);
    if player.watching {
        player.health = theme::FULL_HEALTH;
        player.food = theme::FULL_FOOD;
        player.air = theme::DROWN_SECONDS;
        player.fall_from = player.position.y;
        return;
    }

    player.food = (player.food - theme::FOOD_BURN * delta_time).max(0.0);
    if player.food >= theme::FOOD_HEAL && player.health < theme::FULL_HEALTH {
        player.health = (player.health + theme::HEAL_RATE * delta_time).min(theme::FULL_HEALTH);
        player.food = (player.food - theme::FOOD_BURN * delta_time * 3.0).max(0.0);
    } else if player.food <= 0.0 {
        player.health -= theme::STARVE_RATE * delta_time;
    }

    let head = player.position + Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
    let block = IVec3::new(
        head.x.floor() as i32,
        head.y.floor() as i32,
        head.z.floor() as i32,
    );
    if world
        .blocks
        .fill(block_at(&world, block).index())
        .is_some_and(|fill| fill.liquid == theme::WATER)
    {
        player.air -= delta_time;
        if player.air <= 0.0 {
            player.health -= theme::DROWN_RATE * delta_time;
            player.hurt = theme::HURT_SHAKE;
        }
    } else {
        player.air = theme::DROWN_SECONDS;
    }

    if player.flying || player.boat.is_some() || player.in_water {
        player.fall_from = player.position.y;
    } else if player.on_ground {
        let drop = player.fall_from - player.position.y;
        if drop > theme::FALL_FLOOR {
            player.health -= (drop - theme::FALL_FLOOR) * theme::FALL_HURT;
            player.hurt = theme::HURT_SHAKE;
        }
        player.fall_from = player.position.y;
    } else if player.position.y > player.fall_from {
        player.fall_from = player.position.y;
    }

    if player.health <= 0.0 {
        player.health = theme::FULL_HEALTH;
        player.food = theme::FULL_FOOD * 0.6;
        player.air = theme::DROWN_SECONDS;
        player.position = player.home;
        player.velocity = Vec3::zeros();
        player.fall_from = player.home.y;
        player.boat = None;
        toast_show(&mut toast, "You woke up back at the village");
    }
}
