use crate::contract::WorldSide;
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::{MashupResources, Phase};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::*;

pub fn look(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    mut window: ResMut<Window>,
    mut resources: ResMut<MashupResources>,
) {
    resources.grabbed_now = false;
    if !window.cursor_locked && input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        set_pointer_grab(&mut window, true);
        resources.grabbed_now = true;
        return;
    }
    if !awake(&resources) {
        return;
    }

    let stick = actions.axis2(Ask::Look) * data::STICK_LOOK_SPEED * time.delta_time.min(0.05);

    let turn = if window.cursor_locked {
        input.mouse.raw_mouse_delta * data::MOUSE_SENSITIVITY
    } else {
        Vec2::zeros()
    };

    let quarry = &mut resources.quarry;
    quarry.yaw -= turn.x + stick.x;
    quarry.pitch = (quarry.pitch - turn.y + stick.y).clamp(-data::PITCH_LIMIT, data::PITCH_LIMIT);
}

pub fn walk(time: Res<Time>, actions: Res<Actions<Ask>>, mut resources: ResMut<MashupResources>) {
    if !awake(&resources) {
        return;
    }
    let delta_time = time.delta_time.min(0.05);
    let quarry = &mut resources.quarry;

    let forward = Vec3::new(-quarry.yaw.sin(), 0.0, -quarry.yaw.cos());
    let right = Vec3::new(quarry.yaw.cos(), 0.0, -quarry.yaw.sin());
    let asked = actions.axis2(Ask::Walk);
    let mut wish = right * asked.x + forward * asked.y;
    if wish.norm() > 1.0e-4 {
        wish = wish.normalize() * data::QUARRY_SPEED;
    }

    let jumped = actions.just_pressed(Ask::Jump);
    if jumped && quarry.grounded {
        quarry.velocity.y = data::QUARRY_JUMP;
        quarry.grounded = false;
    }

    quarry.velocity.x = wish.x;
    quarry.velocity.z = wish.z;
    quarry.velocity.y =
        (quarry.velocity.y - data::QUARRY_GRAVITY * delta_time).max(-data::TERMINAL_FALL);

    let slices = queries::slices_for(quarry.velocity, delta_time);
    let slice = delta_time / slices as f32;
    for _ in 0..slices {
        let mut moved = quarry.position;
        for axis in [0_usize, 2] {
            let mut step = moved;
            step[axis] += quarry.velocity[axis] * slice;
            if queries::blocked(moved, step) {
                quarry.velocity[axis] = 0.0;
            } else {
                moved = step;
            }
        }
        moved.y += quarry.velocity.y * slice;
        let ground = queries::standing_height(moved);
        if moved.y <= ground {
            moved.y = ground;
            quarry.velocity.y = 0.0;
            quarry.grounded = true;
        } else {
            quarry.grounded = false;
        }
        quarry.position = moved;
    }
}

fn awake(resources: &MashupResources) -> bool {
    resources.here == WorldSide::Quarry && resources.crossing.phase == Phase::Idle
}
