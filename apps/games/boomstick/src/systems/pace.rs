use crate::data;
use nightshade::prelude::*;
use nightshade_physics::commands::physics_world_set_timestep;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::contract::time::Time;

pub fn steady(time: Res<Time>, mut physics: ResMut<PhysicsWorld>) {
    let step = time
        .delta_time
        .clamp(data::PHYSICS_STEP_SHORTEST, data::PHYSICS_STEP_LONGEST);
    physics_world_set_timestep(&mut physics, step, data::PHYSICS_SUBSTEPS);
}
