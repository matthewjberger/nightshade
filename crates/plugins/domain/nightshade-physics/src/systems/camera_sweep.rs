use crate::resources::PhysicsWorld;
use nightshade::ecs::system_param::{Commands, Query, Res};
use nightshade_camera_controller::contract::{PendingCameraSweep, SweptDistance};

pub fn answer_camera_sweep_system(
    mut commands: Commands,
    physics: Res<PhysicsWorld>,
    asks: Query<(&PendingCameraSweep,)>,
) {
    asks.for_each(|camera, (ask,)| {
        let distance =
            crate::queries::camera_collision::camera_collision_distance(&physics, &ask.ask);
        commands.remove::<(PendingCameraSweep,)>(camera);
        commands.set(camera, SweptDistance { distance });
    });
}
