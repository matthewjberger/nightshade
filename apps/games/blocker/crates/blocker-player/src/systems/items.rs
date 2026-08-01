use crate::components::{Age, Count, Held, Motion, Phase, Velocity};
use crate::data::settings::Settings;
use crate::theme;
use blocker_blocks::queries::{item_scale, slide_box};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::Eye;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::commands::inventory_add;
use blocker_pocket::resources::Inventory;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn drift(
    mut inventory: ResMut<Inventory>,
    world: Res<VoxelWorld>,
    time: Res<Time>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut commands: Commands,
    mut bodies: Query<(&Eye, &Motion)>,
    mut dropped: Query<(
        &Held,
        &mut Count,
        &mut Velocity,
        &mut Age,
        &Phase,
        &mut LocalTransform,
    )>,
) {
    let delta_time = time.delta_time.min(0.05);
    let Some((eye, motion)) = read_the_hero(&hero, &mut bodies, |(eye, motion)| (eye.0, *motion))
    else {
        return;
    };
    let pickup_radius = config.number(Settings::PickupRadius);
    let item_lifetime = config.number(Settings::ItemLifetime);
    let chest = eye - Vec3::new(0.0, theme::EYE_HEIGHT - theme::PLAYER_HEIGHT * 0.5, 0.0);
    let mut gone = Vec::new();
    dropped.for_each_mut(|entity, (held, count, carried, age, phase, transform)| {
        age.0 += delta_time;
        let radius = item_scale(held.0) * 0.5;
        let mut velocity = carried.0;
        velocity.y -= 18.0 * delta_time;
        let slide = slide_box(
            &world,
            transform.translation,
            Vec3::repeat(-radius),
            Vec3::repeat(radius),
            velocity * delta_time,
        );
        let mut position = slide.position;
        if slide.blocked[1] {
            velocity.y = 0.0;
            velocity.x *= 0.7;
            velocity.z *= 0.7;
        }
        for axis in [0, 2] {
            if slide.blocked[axis] {
                velocity[axis] = 0.0;
            }
        }
        if position.y < 1.0 {
            position.y = 1.0 + radius;
            velocity.y = 0.0;
        }
        carried.0 = velocity;
        let bob = ((age.0 + phase.0) * 2.0).sin() * 0.05;
        transform.translation = position + Vec3::new(0.0, bob, 0.0);
        transform.rotation = quat_angle_axis(age.0 * 1.6 + phase.0, &Vec3::y());

        let reach = (chest - position).norm();
        let caught =
            motion != Motion::Watch && age.0 > theme::PICKUP_DELAY && reach < pickup_radius;
        if caught {
            let leftover = inventory_add(&mut inventory, held.0, count.0);
            if leftover == 0 {
                gone.push(entity);
            } else {
                count.0 = leftover;
            }
        } else if age.0 > item_lifetime {
            gone.push(entity);
        }
    });
    if !gone.is_empty() {
        commands.queue(move |storage, resources| {
            despawn_entities_with_cache_cleanup(storage, resources, &gone);
        });
    }
}
