use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_physics::queries::physics_world_cast_ray;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::GlobalTransform;

pub fn follow_the_look(
    physics: Res<PhysicsWorld>,
    active_camera: Res<ActiveCamera>,
    time: Res<Time>,
    mut settings: ResMut<RenderSettings>,
    mut cameras: Query<(&GlobalTransform,)>,
) {
    if !settings.depth_of_field.enabled {
        return;
    }
    let Some(camera) = active_camera.0 else {
        return;
    };
    let mut look = None;
    cameras.for_entity(camera, |(transform,)| {
        let ahead = -transform.0.column(2).xyz();
        look = Some((transform.translation(), ahead.normalize()));
    });
    let Some((from, ahead)) = look else {
        return;
    };
    let wanted = physics_world_cast_ray(&physics, from, ahead, theme::FOCUS_FARTHEST, None)
        .map(|hit| hit.distance)
        .unwrap_or(theme::FOCUS_FARTHEST)
        .max(theme::FOCUS_NEAREST);
    let field = &mut settings.depth_of_field;
    let blend = 1.0 - (-theme::FOCUS_RATE * time.delta_time).exp();
    field.focus_distance += (wanted - field.focus_distance) * blend;
    field.focus_range = (field.focus_distance * theme::FOCUS_RANGE_SHARE)
        .clamp(theme::FOCUS_RANGE_LEAST, theme::FOCUS_RANGE_MOST);
}
