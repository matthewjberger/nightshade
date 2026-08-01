use crate::components::{Heading, Walking};
use crate::resources::TopDownSettings;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_input::contract::{Actions, EngineAction};
use nightshade_platform::contract::time::Time;
use nightshade_scene::contract::LocalTransform;

pub fn walk(
    time: Res<Time>,
    actions: Res<Actions<EngineAction>>,
    settings: Res<TopDownSettings>,
    walkers: Query<(&mut LocalTransform, &Walking, &mut Heading)>,
) {
    let asked = actions.axis2(EngineAction::Move);
    let wanted = vec2(asked.x, -asked.y);
    let delta = time.delta_time;
    walkers.for_each(|_entity, (transform, walking, heading)| {
        if wanted.norm() <= 1.0e-4 {
            return;
        }
        let wanted = wanted.normalize();
        heading.0 = wanted;
        let speed = settings.speed * walking.0;
        transform.translation.x += wanted.x * speed * delta;
        transform.translation.z += wanted.y * speed * delta;
    });
}
