use crate::components::{Boat, Grounded, Riding, Submerged, Velocity, Yaw};
use crate::data::ask::Ask;
use crate::data::settings::Settings;
use crate::theme;
use blocker_blocks::queries::{block_at, liquid_surface_near};
use blocker_blocks::resources::VoxelWorld;
use nalgebra_glm::{IVec3, Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn sail_the_boats(
    world: Res<VoxelWorld>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut boats: Query<(&mut Boat, &mut LocalTransform)>,
    mut sailors: Query<(
        &Riding,
        &Yaw,
        &mut Velocity,
        &mut Grounded,
        &mut Submerged,
        &mut LocalTransform,
    )>,
) {
    let delta_time = time.delta_time.min(0.05);
    let boat_speed = config.number(Settings::BoatSpeed);
    let mut sailing = None;
    sailors.for_each_mut(
        |_, (riding, yaw, velocity, grounded, submerged, transform)| {
            let (sin_yaw, cos_yaw) = yaw.0.sin_cos();
            let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
            let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);
            let steer = actions.axis2(Ask::Move);
            let mut wish = right * steer.x + forward * steer.y;
            if wish.norm_squared() > 0.0 {
                wish = wish.normalize();
            }
            let wanted = wish * boat_speed;
            let ease = (theme::BOAT_EASE * delta_time).min(1.0);
            velocity.0.x += (wanted.x - velocity.0.x) * ease;
            velocity.0.z += (wanted.z - velocity.0.z) * ease;
            velocity.0.y = 0.0;

            let mut position = transform.translation;
            let step = Vec3::new(velocity.0.x * delta_time, 0.0, velocity.0.z * delta_time);
            let wanted_place = position + step;
            let ahead = IVec3::new(
                wanted_place.x.floor() as i32,
                position.y.floor() as i32,
                wanted_place.z.floor() as i32,
            );
            if block_at(&world, ahead).info().solid {
                velocity.0 = Vec3::zeros();
            } else {
                position.x = wanted_place.x;
                position.z = wanted_place.z;
            }
            if let Some(surface) = liquid_surface_near(&world, position) {
                position.y = surface - theme::BOAT_FLOAT;
            }
            transform.translation = position;
            grounded.0 = true;
            submerged.0 = false;
            sailing = Some((riding.0, position, yaw.0));
        },
    );

    if let Some((boat, place, yaw)) = sailing {
        let spin = quat_angle_axis(yaw, &Vec3::y());
        boats.for_entity(boat, |(sailed, transform)| {
            sailed.0 = yaw;
            transform.translation = place;
            transform.rotation = spin;
        });
    }
    let riding = sailing.map(|(boat, _, _)| boat);
    boats.for_each_mut(|entity, (_, transform)| {
        if Some(entity) == riding {
            return;
        }
        if let Some(surface) = liquid_surface_near(&world, transform.translation) {
            transform.translation.y = surface - theme::BOAT_FLOAT;
        }
    });
}
