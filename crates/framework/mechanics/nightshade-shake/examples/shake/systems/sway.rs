use crate::components::Rest;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::ecs::system_param::Query;
use nightshade_scene::contract::LocalTransform;
use nightshade_shake::{ShakeOffset, ShakeTurn};

pub fn sway_the_posts(posts: Query<(&mut LocalTransform, &Rest, &ShakeOffset, &ShakeTurn)>) {
    posts.for_each(|_entity, (transform, rest, offset, turn)| {
        transform.translation = rest.0 + offset.0;
        transform.rotation = quat_angle_axis(turn.0, &Vec3::z_axis());
    });
}
