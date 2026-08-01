use crate::components::Boat;
use crate::data::Ask;
use crate::queries::{block_at, liquid_surface_near};
use crate::resources::{Player, VoxelWorld};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn sail_the_boats(
    mut player: ResMut<Player>,
    world: Res<VoxelWorld>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    mut boats: Query<(&mut Boat, &mut LocalTransform)>,
) {
    let riding = player.boat;
    let delta_time = time.delta_time.min(0.05);
    if let Some(riding) = riding {
        let (sin_yaw, cos_yaw) = player.yaw.sin_cos();
        let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
        let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);
        let walk = actions.axis2(Ask::Move);
        let mut wish = right * walk.x + forward * walk.y;
        if wish.norm_squared() > 0.0 {
            wish = wish.normalize();
        }
        let wanted = wish * theme::BOAT_SPEED;
        let ease = (theme::BOAT_EASE * delta_time).min(1.0);
        player.velocity.x += (wanted.x - player.velocity.x) * ease;
        player.velocity.z += (wanted.z - player.velocity.z) * ease;
        player.velocity.y = 0.0;

        let step = Vec3::new(
            player.velocity.x * delta_time,
            0.0,
            player.velocity.z * delta_time,
        );
        let wanted_place = player.position + step;
        let ahead = IVec3::new(
            wanted_place.x.floor() as i32,
            player.position.y.floor() as i32,
            wanted_place.z.floor() as i32,
        );
        if block_at(&world, ahead).info().solid {
            player.velocity = Vec3::zeros();
        } else {
            player.position.x = wanted_place.x;
            player.position.z = wanted_place.z;
        }
        if let Some(surface) = liquid_surface_near(&world, player.position) {
            player.position.y = surface - theme::BOAT_FLOAT;
        }
        player.on_ground = true;
        player.in_water = false;

        let place = player.position;
        let spin = quat_angle_axis(player.yaw, &Vec3::y());
        boats.for_entity(riding, |(boat, transform)| {
            boat.yaw = player.yaw;
            transform.translation = place;
            transform.rotation = spin;
        });
    }
    boats.for_each_mut(|entity, (_, transform)| {
        if Some(entity) == riding {
            return;
        }
        if let Some(surface) = liquid_surface_near(&world, transform.translation) {
            transform.translation.y = surface - theme::BOAT_FLOAT;
        }
    });
}
