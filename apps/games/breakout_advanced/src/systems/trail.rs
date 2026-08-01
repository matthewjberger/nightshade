use crate::components::{Fade, Radius, Velocity};
use crate::data;
use crate::queries;
use crate::resources::BreakoutAdvancedResources;
use crate::systems::setup::BALL_MATERIAL;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn shed(
    time: Res<Time>,
    mut resources: ResMut<BreakoutAdvancedResources>,
    mut commands: Commands,
    mut balls: Query<(&Radius, &Velocity, &LocalTransform)>,
) {
    let delta_time = time.delta_time * resources.juice.time_scale;
    let mut pieces = Vec::new();

    let debt = resources.juice.trail_debt;
    let mut carried = debt;
    balls.for_each_mut(|_entity, (radius, velocity, transform)| {
        let travel = velocity.0.magnitude() * delta_time;
        carried = debt + travel;
        while carried >= data::TRAIL_STEP {
            carried -= data::TRAIL_STEP;
            pieces.push((transform.translation, radius.0));
        }
    });
    resources.juice.trail_debt = carried;

    for (at, radius) in pieces {
        let entity = spawn_mesh_at(
            &mut commands,
            "Sphere",
            at,
            Vec3::new(radius, radius, radius) * data::TRAIL_SCALE,
        );
        set_mesh_material(&mut commands, entity, BALL_MATERIAL);
        commands.set(entity, Fade(data::TRAIL_LIFE));
    }
}

pub fn fade(
    time: Res<Time>,
    mut commands: Commands,
    pieces: Query<(&mut Fade, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let mut spent = Vec::new();
    pieces.for_each(|entity, (remaining, transform)| {
        remaining.0 -= delta_time;
        if remaining.0 <= 0.0 {
            spent.push(entity);
            return;
        }
        let share = queries::fade_scale(remaining.0) * data::BALL_RADIUS * data::TRAIL_SCALE;
        transform.scale = Vec3::new(share, share, share);
    });
    for entity in spent {
        commands.despawn(entity);
    }
}
