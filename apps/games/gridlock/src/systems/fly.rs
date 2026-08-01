use crate::data::ask::Ask;
use crate::data::settings::Settings;
use crate::queries::view::{look_angles, look_rotation};
use crate::resources::Player;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

const PITCH_LIMIT: f32 = 1.5;
const GAMEPAD_LOOK: f32 = 2.4;
const SPRINT_FACTOR: f32 = 3.0;

pub fn toggle(
    actions: Res<Actions<Ask>>,
    mut player: ResMut<Player>,
    mut active_camera: ResMut<ActiveCamera>,
    storage: &mut Storage,
) {
    if !actions.just_pressed(Ask::Fly) {
        return;
    }
    let (Some(fly), Some(walk), Some(chase)) =
        (player.fly_camera, player.walk_camera, player.chase_camera)
    else {
        return;
    };
    if player.flying {
        player.flying = false;
        active_camera.0 = Some(if player.car.is_some() { chase } else { walk });
        return;
    }
    let Some(from) = active_camera
        .0
        .and_then(|camera| storage.get::<LocalTransform>(camera))
        .map(|transform| (transform.translation, transform.rotation))
    else {
        return;
    };
    let (yaw, pitch) = look_angles(&from.1);
    if let Some(transform) = storage.get_mut::<LocalTransform>(fly) {
        transform.translation = from.0;
        transform.rotation = look_rotation(yaw, pitch);
    }
    player.fly_yaw = yaw;
    player.fly_pitch = pitch;
    player.flying = true;
    active_camera.0 = Some(fly);
}

pub fn run(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    mut player: ResMut<Player>,
    mut cameras: Query<(&mut LocalTransform,)>,
) {
    if !player.flying {
        return;
    }
    let Some(fly) = player.fly_camera else {
        return;
    };
    let delta_time = time.delta_time;
    let look = config.number(Settings::FlyLook);
    if input.mouse.state.contains(MouseState::RIGHT_CLICKED) {
        let delta = input.mouse.position_delta;
        player.fly_yaw -= delta.x * look;
        player.fly_pitch -= delta.y * look;
    }
    let stick = actions.axis2(Ask::Look);
    player.fly_yaw -= stick.x * GAMEPAD_LOOK * delta_time;
    player.fly_pitch -= stick.y * GAMEPAD_LOOK * delta_time;
    player.fly_pitch = player.fly_pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

    let walk = actions.axis2(Ask::Walk);
    let rise = actions.axis(Ask::Rise);
    let fast = actions.pressed(Ask::Sprint);
    let rotation = look_rotation(player.fly_yaw, player.fly_pitch);
    let forward = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    let right = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(1.0, 0.0, 0.0));
    let speed = config.number(Settings::FlySpeed) * if fast { SPRINT_FACTOR } else { 1.0 };
    let motion = (forward * walk.y + right * walk.x + Vec3::y() * rise) * speed * delta_time;
    cameras.for_entity(fly, |(transform,)| {
        transform.translation += motion;
        transform.translation.y = transform.translation.y.max(0.5);
        transform.rotation = rotation;
    });
}
