use crate::commands::next_signed;
use crate::components::{Angle, Cooldown, Speed};
use crate::contract::NeonStaticEvent;
use crate::data;
use crate::data::Ask;
use crate::queries::{blocked, pointer_world_position, viewport_rectangle};
use crate::resources::{FloorPhase, NeonStaticResources};
use crate::systems::setup::{spawn_bullet, spawn_shell};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::*;

pub fn drive(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    mut resources: ResMut<NeonStaticResources>,
    mut events: EventWriter<NeonStaticEvent>,
    mut commands: Commands,
    players: Query<(&Speed, &mut Angle, &mut Cooldown, &mut LocalTransform)>,
) {
    let resources = &mut *resources;
    let Some((origin, size)) = viewport_rectangle(&viewport, &window) else {
        return;
    };
    let pointer = pointer_world_position(
        origin,
        size,
        resources.camera_position,
        input.mouse.position,
    );
    let delta_time = time.delta_time;
    let walking = resources.floor.phase == FloorPhase::Fighting;
    let direction = actions.axis2(Ask::Walk);
    let firing = walking && actions.pressed(Ask::Fire);

    let mut here = resources.player_position;
    let mut fired = None;
    {
        let floor = &resources.floor;
        players.for_each(|_entity, (speed, angle, cooldown, transform)| {
            cooldown.0 = (cooldown.0 - delta_time).max(0.0);
            let mut position = Vec2::new(transform.translation.x, transform.translation.y);
            if walking {
                let step = direction * speed.0 * delta_time;
                let sideways = Vec2::new(position.x + step.x, position.y);
                if !blocked(floor, sideways, data::BODY_RADIUS) {
                    position.x = sideways.x;
                }
                let forward = Vec2::new(position.x, position.y + step.y);
                if !blocked(floor, forward, data::BODY_RADIUS) {
                    position.y = forward.y;
                }
            }

            let toward = pointer - position;
            if toward.magnitude_squared() > 1.0e-6 {
                angle.0 = toward.y.atan2(toward.x);
            }
            transform.translation = Vec3::new(position.x, position.y, 0.0);
            transform.rotation = nalgebra_glm::quat_angle_axis(angle.0, &Vec3::z());
            here = position;

            if firing && cooldown.0 <= 0.0 {
                cooldown.0 = data::FIRE_INTERVAL;
                fired = Some((position, Vec2::new(angle.0.cos(), angle.0.sin())));
            }
        });
    }
    resources.player_position = here;

    let Some((at, aim)) = fired else {
        return;
    };
    let muzzle = at + aim * data::MUZZLE_REACH;
    spawn_bullet(&resources.sheets, muzzle, aim, &mut commands);

    let side = Vec2::new(-aim.y, aim.x);
    let scatter = next_signed(&mut resources.floor);
    let spin = next_signed(&mut resources.floor) * data::SHELL_SPIN;
    let toss = side * (data::SHELL_TOSS + scatter) - aim * data::SHELL_DRIFT;
    spawn_shell(&resources.sheets, at, toss, spin, &mut commands);

    events.send(NeonStaticEvent::ShotFired { at: muzzle });
}
