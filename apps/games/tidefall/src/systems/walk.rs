use crate::components::{Bulk, Fuel, Ground, Grounded, Pitch, Velocity};
use crate::data::{
    AIR_CONTROL, Ask, GAMEPAD_LOOK, GRIP_RATE, LOOK_RADIANS_PER_PIXEL, MAX_STEP, REFUEL_SECONDS,
    SLOPE_STAND, Settings, WALK_PITCH_LIMIT, WALKER_RADIUS,
};
use crate::queries::{
    Bodies, Drafts, Features, collect_bodies, collect_colliders, collect_drafts, draft_at,
    nearest_body, surface_gravity,
};
use crate::resources::{Mode, TidefallResources};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn walk(
    time: Res<Time>,
    input: Res<Input>,
    window: Res<Window>,
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    resources: Res<TidefallResources>,
    bodies: Bodies<'_, '_>,
    features: Features<'_, '_>,
    drafts: Drafts<'_, '_>,
    mut ships: Query<(&Bulk, &LocalTransform)>,
    mut walkers: Query<(
        &mut Velocity,
        &mut Ground,
        &mut Grounded,
        &mut Pitch,
        &mut Fuel,
        &mut LocalTransform,
    )>,
) {
    if resources.mode != Mode::Afoot {
        return;
    }
    let (Some(ship), Some(walker)) = (resources.ship, resources.walker) else {
        return;
    };
    let delta_time = time.delta_time.min(MAX_STEP);
    if delta_time <= f32::EPSILON {
        return;
    }
    let views = collect_bodies(bodies);
    let colliders = collect_colliders(features, &views);
    let updrafts = collect_drafts(drafts, &views);
    let mut hull = None;
    ships.for_entity(ship, |(bulk, transform)| {
        hull = Some((transform.translation, bulk.0));
    });

    let mouse = if window.cursor_locked {
        input.mouse.raw_mouse_delta * LOOK_RADIANS_PER_PIXEL * config.number(Settings::Look)
    } else {
        Vec2::zeros()
    };
    let stick = actions.axis2(Ask::Look) * GAMEPAD_LOOK * delta_time;
    let turn = mouse + Vec2::new(stick.x, -stick.y);
    let stride = actions.axis2(Ask::Move);
    let lifting = actions.axis(Ask::Lift) > 0.0 || actions.pressed(Ask::Jump);
    let jumping = actions.just_pressed(Ask::Jump);
    let walk_speed = config.number(Settings::Walk);
    let jump_speed = config.number(Settings::Jump);
    let jet_thrust = config.number(Settings::Jetpack);
    let fuel_seconds = config.number(Settings::Fuel).max(0.1);
    let gravity_scale = config.number(Settings::Gravity);
    let clock = config.number(Settings::Clock);

    walkers.for_entity(
        walker,
        |(velocity, ground, grounded, pitch, fuel, transform)| {
            let Some(index) = nearest_body(transform.translation, &views) else {
                return;
            };
            let view = &views[index];
            ground.0 = Some(view.entity);

            let axis = nalgebra_glm::quat_rotate_vec3(&view.rotation, &Vec3::y());
            let spin = nalgebra_glm::quat_angle_axis(view.spin * delta_time * clock, &axis);
            let carried = transform.translation + view.motion * delta_time;
            let mut offset = nalgebra_glm::quat_rotate_vec3(&spin, &(carried - view.center));
            let up = offset.normalize();

            let mut body = nalgebra_glm::quat_normalize(&(spin * transform.rotation));
            let current_up = nalgebra_glm::quat_rotate_vec3(&body, &Vec3::y());
            let align = nalgebra_glm::quat_rotation(&current_up, &up);
            body = nalgebra_glm::quat_normalize(&(align * body));
            body =
                nalgebra_glm::quat_normalize(&(nalgebra_glm::quat_angle_axis(-turn.x, &up) * body));
            pitch.0 = (pitch.0 - turn.y).clamp(-WALK_PITCH_LIMIT, WALK_PITCH_LIMIT);
            transform.rotation = nalgebra_glm::quat_normalize(
                &(body * nalgebra_glm::quat_angle_axis(pitch.0, &Vec3::x())),
            );

            let forward = nalgebra_glm::quat_rotate_vec3(&body, &Vec3::new(0.0, 0.0, -1.0));
            let right = nalgebra_glm::quat_rotate_vec3(&body, &Vec3::x());
            let wish = (forward * stride.y + right * stride.x) * walk_speed;

            if grounded.0 {
                let radial = velocity.0.dot(&up);
                let along = velocity.0 - up * radial;
                let blend = (GRIP_RATE * view.grip * delta_time).min(1.0);
                velocity.0 = along + (wish - along) * blend + up * radial;
                if jumping {
                    velocity.0 += up * jump_speed;
                }
                fuel.0 = (fuel.0 + delta_time / REFUEL_SECONDS).min(1.0);
            } else {
                let radial = velocity.0.dot(&up);
                let along = velocity.0 - up * radial;
                let steer = wish - along;
                let reach = steer.magnitude();
                if reach > f32::EPSILON {
                    velocity.0 += steer / reach * (AIR_CONTROL * delta_time).min(reach);
                }
                if lifting && fuel.0 > 0.0 {
                    velocity.0 += up * jet_thrust * delta_time;
                    fuel.0 = (fuel.0 - delta_time / fuel_seconds).max(0.0);
                }
            }
            velocity.0 -= up * surface_gravity(view) * gravity_scale * delta_time;
            velocity.0 += draft_at(transform.translation, view.entity, &updrafts) * delta_time;
            offset += velocity.0 * delta_time;

            let mut standing = false;
            let distance = offset.magnitude();
            if distance < view.radius + WALKER_RADIUS {
                let normal = offset / distance;
                offset = normal * (view.radius + WALKER_RADIUS);
                let into = velocity.0.dot(&normal);
                if into < 0.0 {
                    velocity.0 -= normal * into;
                }
                standing = true;
            }
            let mut position = view.center + offset;
            let mut obstacles: Vec<(Vec3, f32)> = colliders
                .iter()
                .filter(|collider| collider.body == view.entity)
                .map(|collider| (collider.center, collider.radius))
                .collect();
            if let Some(hull) = hull {
                obstacles.push(hull);
            }
            for (center, radius) in obstacles {
                let gap = position - center;
                let distance = gap.magnitude();
                if distance < radius + WALKER_RADIUS && distance > f32::EPSILON {
                    let normal = gap / distance;
                    position += normal * (radius + WALKER_RADIUS - distance);
                    let into = velocity.0.dot(&normal);
                    if into < 0.0 {
                        velocity.0 -= normal * into;
                    }
                    if normal.dot(&up) > SLOPE_STAND {
                        standing = true;
                    }
                }
            }
            grounded.0 = standing;
            transform.translation = position;
        },
    );
}
