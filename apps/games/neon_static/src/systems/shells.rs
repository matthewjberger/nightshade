use crate::components::{Angle, Life, Spin, Velocity};
use crate::data;
use crate::queries::solid_at;
use crate::resources::NeonStaticResources;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;

pub fn settle(
    time: Res<Time>,
    resources: Res<NeonStaticResources>,
    mut commands: Commands,
    shells: Query<(
        &mut Velocity,
        &mut Spin,
        &mut Angle,
        &mut Life,
        &mut LocalTransform,
    )>,
) {
    let delta_time = time.delta_time;
    let floor = &resources.floor;
    let drag = (1.0 - data::SHELL_DRAG * delta_time).clamp(0.0, 1.0);
    let mut spent = Vec::new();

    shells.for_each(|entity, (velocity, spin, angle, life, transform)| {
        life.0 -= delta_time;
        if life.0 <= 0.0 {
            spent.push(entity);
            return;
        }
        velocity.0 *= drag;
        spin.0 *= drag;
        angle.0 += spin.0 * delta_time;
        let position =
            Vec2::new(transform.translation.x, transform.translation.y) + velocity.0 * delta_time;
        if !solid_at(floor, position) {
            transform.translation = Vec3::new(position.x, position.y, 0.0);
        }
        transform.rotation = nalgebra_glm::quat_angle_axis(angle.0, &Vec3::z());
    });

    for entity in spent {
        commands.despawn(entity);
    }
}
