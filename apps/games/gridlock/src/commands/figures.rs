use crate::commands::models::transform_at;
use crate::components::{
    Bend, Came, Cell, Corner, Cruise, Dying, Fit, Heading, Idle, Look, Next, Pace, Phase, Rig,
    Steer, Stride, Velocity, Yaw,
};
use crate::data::districts::HERO;
use crate::data::scale::{CAR_SCALE, FOLK_SCALE};
use crate::queries::grid::{corner_position, heading_vector, intersection_ahead, yaw_of};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_kit::prelude::{MODELS, model};
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn spawn_hero(commands: &mut Commands, position: Vec3) -> Entity {
    commands.spawn((
        Name("Hero".to_string()),
        LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Look(model(HERO)),
        Fit(FOLK_SCALE),
        Rig(None),
        Velocity::default(),
        Yaw::default(),
        Stride::default(),
        Phase::default(),
    ))
}

pub fn spawn_person(
    storage: &mut Storage,
    look: usize,
    node: [i32; 3],
    next: [i32; 3],
    progress: f32,
    pace: f32,
    cell: [i32; 2],
) -> Entity {
    let start = corner_position(node);
    let heading = corner_position(next) - start;
    let position = start + heading * progress;
    storage.spawn((
        Name(MODELS[look].name.to_string()),
        transform_at(position, yaw_of(heading)),
        GlobalTransform::default(),
        Look(look),
        Fit(FOLK_SCALE),
        Cell(cell),
        Rig(None),
        Stride::default(),
        Phase::default(),
        Pace(pace),
        Corner(next),
        Came(node),
        Idle::default(),
        Dying::default(),
    ))
}

pub fn spawn_car(
    storage: &mut Storage,
    look: usize,
    position: Vec3,
    heading: u8,
    cruise: f32,
    cell: [i32; 2],
) -> Entity {
    let direction = heading_vector(heading);
    storage.spawn((
        Name(MODELS[look].name.to_string()),
        transform_at(position, yaw_of(direction)),
        GlobalTransform::default(),
        Look(look),
        Fit(CAR_SCALE),
        Cell(cell),
        Velocity::default(),
        Yaw(yaw_of(direction)),
        Steer::default(),
        Heading(heading),
        Next(intersection_ahead(position, heading)),
        Bend(-1.0),
        Cruise(cruise),
    ))
}
