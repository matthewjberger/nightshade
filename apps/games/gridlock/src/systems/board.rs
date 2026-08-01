use crate::components::{Bend, Heading, Next, Turn, Velocity, Yaw};
use crate::contract::GridlockEvent;
use crate::data::ask::Ask;
use crate::data::motion::CAR_SEAT;
use crate::data::scale::CAR_SCALE;
use crate::data::settings::Settings;
use crate::queries::grid::{heading_of_yaw, intersection_ahead, right_of};
use crate::resources::{Indoors, Player};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_kit::prelude::MODELS;
use nightshade_scene::prelude::{ChildOf, LocalTransform, Name};

const EXIT_SIDE: f32 = 2.8;

pub fn update(
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    indoors: Res<Indoors>,
    mut player: ResMut<Player>,
    mut active_camera: ResMut<ActiveCamera>,
    mut events: EventWriter<GridlockEvent>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    if !actions.just_pressed(Ask::Use) || player.flying || indoors.used || indoors.visit.is_some() {
        return;
    }
    let Some(body) = player.body else {
        return;
    };

    if let Some(car) = player.car {
        let Some((position, rotation, yaw)) = storage
            .get::<LocalTransform>(car)
            .map(|transform| (transform.translation, transform.rotation, transform))
            .and_then(|(position, rotation, _)| {
                storage
                    .get::<Yaw>(car)
                    .map(|yaw| (position, rotation, yaw.0))
            })
        else {
            return;
        };
        let heading = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        let beside = position + right_of(heading) * EXIT_SIDE;
        commands.remove::<(ChildOf,)>(body);
        commands.set(
            body,
            LocalTransform {
                translation: Vec3::new(beside.x, 0.0, beside.z),
                rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        commands.set(body, Velocity::default());
        commands.set(body, Yaw(yaw));
        let facing = heading_of_yaw(yaw);
        commands.set(car, Heading(facing));
        commands.set(car, Turn(facing));
        commands.set(car, Next(intersection_ahead(position, facing)));
        commands.set(car, Bend(-1.0));
        player.car = None;
        active_camera.0 = player.walk_camera;
        events.send(GridlockEvent::Left {
            car: car_name(storage, car),
        });
        return;
    }

    let Some(feet) = storage
        .get::<LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let reach = config.number(Settings::BoardReach);
    let mut nearest: Option<(f32, Entity)> = None;
    for (entity, (_, transform)) in storage.query_ref::<(&Heading, &LocalTransform)>().iter() {
        let distance = (transform.translation - feet).norm();
        if distance <= reach && nearest.is_none_or(|(best, _)| distance < best) {
            nearest = Some((distance, entity));
        }
    }
    let Some((_, car)) = nearest else {
        return;
    };
    commands.set(body, ChildOf(car));
    commands.set(
        body,
        LocalTransform {
            translation: Vec3::new(
                CAR_SEAT[0] * CAR_SCALE,
                CAR_SEAT[1] * CAR_SCALE,
                CAR_SEAT[2] * CAR_SCALE,
            ),
            rotation: nalgebra_glm::Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(body, Velocity::default());
    commands.set(car, Bend(-1.0));
    player.car = Some(car);
    active_camera.0 = player.chase_camera;
    events.send(GridlockEvent::Boarded {
        car: car_name(storage, car),
    });
}

fn car_name(storage: &Storage, car: Entity) -> String {
    storage
        .get::<Name>(car)
        .map(|name| name.0.trim_start_matches("car-").replace('-', " "))
        .unwrap_or_else(|| MODELS[0].name.to_string())
}
