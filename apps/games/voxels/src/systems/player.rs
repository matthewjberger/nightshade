use crate::commands::set_watching;
use crate::data::Ask;
use crate::queries::{box_in_liquid, player_box, slide_box};
use crate::resources::{Player, VoxelWorld};
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn control(
    mut player: ResMut<Player>,
    world: Res<VoxelWorld>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    window: Res<Window>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let Some(camera) = player.camera else {
        return;
    };
    let delta_time = time.delta_time.min(0.05);

    if window.cursor_locked {
        let look = input.mouse.raw_mouse_delta * theme::MOUSE_SENSITIVITY;
        player.yaw -= look.x;
        player.pitch = (player.pitch - look.y).clamp(-1.55, 1.55);
    }
    let turn = actions.axis2(Ask::Look) * theme::PAD_LOOK_RATE * delta_time;
    player.yaw -= turn.x;
    player.pitch = (player.pitch + turn.y).clamp(-1.55, 1.55);

    if player.boat.is_some() {
        let rotation =
            quat_angle_axis(player.yaw, &Vec3::y()) * quat_angle_axis(player.pitch, &Vec3::x());
        let eye = player.position + Vec3::new(0.0, theme::EYE_HEIGHT * 0.8, 0.0);
        player.eye = eye;
        player.forward = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
        cameras.for_entity(camera, |(_, transform)| {
            transform.translation = eye;
            transform.rotation = rotation;
        });
        return;
    }
    if actions.just_pressed(Ask::Fly) {
        player.flying = !player.flying;
        player.velocity.y = 0.0;
    }
    if actions.just_pressed(Ask::Watch) {
        let watching = !player.watching;
        set_watching(&mut player, watching);
    }

    let (sin_yaw, cos_yaw) = player.yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);
    let locked = window.cursor_locked;
    let walk = if locked {
        actions.axis2(Ask::Move)
    } else {
        Vec2::zeros()
    };
    let mut wish = right * walk.x + forward * walk.y;
    if wish.norm_squared() > 0.0 {
        wish = wish.normalize();
    }
    let jump = locked && actions.pressed(Ask::Jump);
    let sink = locked && actions.pressed(Ask::Sink);
    let sprint = locked && actions.pressed(Ask::Sprint);

    let (min, max) = player_box(player.position);
    player.in_water =
        box_in_liquid(&world, min, max + Vec3::new(0.0, -0.9, 0.0)) == Some(theme::WATER);

    if player.watching {
        let wheel = input.mouse.wheel_delta.y;
        if locked && wheel.abs() > 0.01 {
            player.watch_speed = (player.watch_speed + wheel.signum() * theme::WATCH_SPEED_STEP)
                .clamp(theme::WATCH_SPEED_MIN, theme::WATCH_SPEED_MAX);
        }
        if player.watched.is_some() && sink {
            player.watched = None;
        }
        let riding = player.watched.is_some();
        let speed = if sprint {
            player.watch_speed * 2.5
        } else {
            player.watch_speed
        };
        let mut velocity = if riding { Vec3::zeros() } else { wish * speed };
        if jump && !riding {
            velocity.y += speed;
        }
        if sink && !riding {
            velocity.y -= speed;
        }
        player.velocity = velocity;
        let mut position = player.position + velocity * delta_time;
        position.y = position
            .y
            .clamp(theme::FLOOR_Y as f32 + 1.0, theme::WORLD_Y as f32 - 1.0);
        player.position = position;
        player.on_ground = false;
        let rotation =
            quat_angle_axis(player.yaw, &Vec3::y()) * quat_angle_axis(player.pitch, &Vec3::x());
        let eye = position + Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
        player.eye = eye;
        player.forward = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
        cameras.for_entity(camera, |(_, transform)| {
            transform.translation = eye;
            transform.rotation = rotation;
        });
        return;
    }

    if player.flying {
        let speed = if sprint {
            theme::FLY_SPEED * 2.0
        } else {
            theme::FLY_SPEED
        };
        let mut velocity = wish * speed;
        if jump {
            velocity.y += speed;
        }
        if sink {
            velocity.y -= speed;
        }
        player.velocity = velocity;
    } else if player.in_water {
        let mut velocity = player.velocity;
        velocity.x = wish.x * theme::SWIM_SPEED;
        velocity.z = wish.z * theme::SWIM_SPEED;
        velocity.y -= theme::WATER_GRAVITY * delta_time;
        velocity.y = velocity.y.max(-3.0);
        if jump {
            velocity.y = 3.2;
        }
        player.velocity = velocity;
    } else {
        let speed = if sprint {
            theme::SPRINT_SPEED
        } else {
            theme::WALK_SPEED
        };
        let mut velocity = player.velocity;
        velocity.x = wish.x * speed;
        velocity.z = wish.z * speed;
        velocity.y -= theme::GRAVITY * delta_time;
        if jump && player.on_ground {
            velocity.y = theme::JUMP_SPEED;
        }
        player.velocity = velocity;
    }

    let low = Vec3::new(-theme::PLAYER_HALF_WIDTH, 0.0, -theme::PLAYER_HALF_WIDTH);
    let high = Vec3::new(
        theme::PLAYER_HALF_WIDTH,
        theme::PLAYER_HEIGHT,
        theme::PLAYER_HALF_WIDTH,
    );
    let slide = slide_box(
        &world,
        player.position,
        low,
        high,
        player.velocity * delta_time,
    );
    for axis in 0..3 {
        if slide.blocked[axis] {
            player.velocity[axis] = 0.0;
        }
    }
    let mut position = slide.position;
    let on_ground = slide.on_ground;
    if position.y < 1.0 {
        position.y = theme::WORLD_Y as f32;
    }
    player.position = position;
    player.on_ground = on_ground;

    let rotation =
        quat_angle_axis(player.yaw, &Vec3::y()) * quat_angle_axis(player.pitch, &Vec3::x());
    let eye = position + Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
    player.eye = eye;
    player.forward = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    cameras.for_entity(camera, |(_, transform)| {
        transform.translation = eye;
        transform.rotation = rotation;
    });
}
