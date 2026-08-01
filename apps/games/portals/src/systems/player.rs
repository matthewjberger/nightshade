use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::PortalsResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn look(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    mut window: ResMut<Window>,
    mut resources: ResMut<PortalsResources>,
) {
    resources.player.grabbed_now = false;

    let stick = actions.axis2(Ask::Look) * data::STICK_LOOK_SPEED * time.delta_time.min(0.05);

    if !window.cursor_locked && input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        set_pointer_grab(&mut window, true);
        resources.player.grabbed_now = true;
    }

    let turn = if window.cursor_locked {
        input.mouse.raw_mouse_delta * data::MOUSE_SENSITIVITY
    } else {
        Vec2::zeros()
    };

    let player = &mut resources.player;
    player.yaw -= turn.x + stick.x;
    player.pitch = (player.pitch - turn.y + stick.y).clamp(-data::PITCH_LIMIT, data::PITCH_LIMIT);
}

pub fn walk(time: Res<Time>, actions: Res<Actions<Ask>>, mut resources: ResMut<PortalsResources>) {
    let delta_time = time.delta_time.min(0.05);
    let resources = &mut *resources;
    let solids = &resources.solids;
    let player = &mut resources.player;

    let forward = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
    let right = Vec3::new(player.yaw.cos(), 0.0, -player.yaw.sin());
    let asked = actions.axis2(Ask::Walk);
    let mut wish = right * asked.x + forward * asked.y;
    if wish.norm() > 1.0e-4 {
        wish = wish.normalize() * data::WALK_SPEED;
    }

    player.velocity.x = wish.x;
    player.velocity.z = wish.z;
    player.velocity.y = (player.velocity.y - data::GRAVITY * delta_time).max(-data::TERMINAL_FALL);

    let slices = queries::slices_for(player.velocity, delta_time);
    let slice = delta_time / slices as f32;
    let mut position = player.position;
    for _ in 0..slices {
        for axis in [0usize, 1, 2] {
            position[axis] += player.velocity[axis] * slice;
            for _ in 0..4 {
                let Some(correction) = queries::shortest_push(position, axis, solids) else {
                    break;
                };
                position[axis] += correction;
                player.velocity[axis] = 0.0;
            }
        }
    }
    player.position = position;
}

pub fn place(resources: Res<PortalsResources>, mut commands: Commands) {
    let player = &resources.player;

    if player.camera != Entity::default() {
        commands.set(
            player.camera,
            LocalTransform {
                translation: queries::eye_position(player),
                rotation: queries::player_rotation(player),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
    }

    if player.avatar != Entity::default() {
        commands.set(
            player.avatar,
            LocalTransform {
                translation: player.position + Vec3::new(0.0, data::PLAYER_HEIGHT * 0.5, 0.0),
                rotation: nalgebra_glm::quat_angle_axis(player.yaw, &Vec3::y()),
                scale: Vec3::new(
                    data::AVATAR_RADIUS * 2.0,
                    data::PLAYER_HEIGHT,
                    data::AVATAR_RADIUS * 2.0,
                ),
            },
        );
    }

    if player.label != Entity::default() {
        commands.set(
            player.label,
            LocalTransform {
                translation: player.position
                    + Vec3::new(0.0, data::PLAYER_HEIGHT + data::LABEL_LIFT, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(data::LABEL_SCALE, data::LABEL_SCALE, data::LABEL_SCALE),
            },
        );
    }
}
