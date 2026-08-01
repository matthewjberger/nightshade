use crate::components::{
    Angle, AngularSpeed, BobPhase, Health, Height, Hit, Label, OrbitRadius, Respawn,
};
use crate::resources::Jarvis;
use crate::theme::DRONE_HEALTH;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;

const FLIGHT_MODE_SPEED: f32 = 3.0;
const BOB_HEIGHT: f32 = 0.25;
const BOB_RATE: f32 = 1.3;
const HIT_FADE: f32 = 2.5;
const RESPAWN_TURN: f32 = 2.1;
const RESPAWN_HEIGHTS: [f32; 6] = [1.2, 2.6, 1.8, 3.3, 1.5, 2.2];

pub fn fly_drones(jarvis: Res<Jarvis>, time: Res<Time>, storage: &mut nightshade::ecs::Storage) {
    let speed_scale = if jarvis.flight_mode {
        FLIGHT_MODE_SPEED
    } else {
        1.0
    };
    let uptime = time.uptime_milliseconds as f32 / 1000.0;
    let kills = jarvis.kills as usize;
    let drones: Vec<Entity> = storage
        .query_entities::<(Label, LocalTransform)>()
        .collect();

    for entity in drones {
        let Some(label) = storage.get::<Label>(entity).map(|label| label.0) else {
            continue;
        };
        let orbit_radius = storage
            .get::<OrbitRadius>(entity)
            .map(|radius| radius.0)
            .unwrap_or_default();
        let angular_speed = storage
            .get::<AngularSpeed>(entity)
            .map(|speed| speed.0)
            .unwrap_or_default();
        let bob_phase = storage
            .get::<BobPhase>(entity)
            .map(|phase| phase.0)
            .unwrap_or_default();
        let mut angle = storage
            .get::<Angle>(entity)
            .map(|angle| angle.0)
            .unwrap_or_default();
        let mut height = storage
            .get::<Height>(entity)
            .map(|height| height.0)
            .unwrap_or_default();
        let mut hit = storage
            .get::<Hit>(entity)
            .map(|hit| hit.0)
            .unwrap_or_default();
        let mut respawn = storage
            .get::<Respawn>(entity)
            .map(|respawn| respawn.0)
            .unwrap_or_default();

        hit = (hit - time.delta_time * HIT_FADE).max(0.0);
        let mut came_back = false;
        if respawn > 0.0 {
            respawn -= time.delta_time;
            if respawn > 0.0 {
                storage.set(entity, Hit(hit));
                storage.set(entity, Respawn(respawn));
                continue;
            }
            respawn = 0.0;
            storage.set(entity, Health(DRONE_HEALTH));
            angle += RESPAWN_TURN + label as f32 * 0.9;
            height = RESPAWN_HEIGHTS[(label + kills) % RESPAWN_HEIGHTS.len()];
            hit = 1.0;
            came_back = true;
        }
        angle += angular_speed * speed_scale * time.delta_time;

        storage.set(entity, Angle(angle));
        storage.set(entity, Height(height));
        storage.set(entity, Hit(hit));
        storage.set(entity, Respawn(respawn));

        let translation = Vec3::new(
            orbit_radius * angle.cos(),
            height + (uptime * BOB_RATE + bob_phase).sin() * BOB_HEIGHT,
            orbit_radius * angle.sin(),
        );
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation = translation;
        }
        if came_back {
            storage.set(entity, Visibility { visible: true });
        }
    }
}
