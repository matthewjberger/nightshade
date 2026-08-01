use crate::components::{Avatar, Fit, Look, Replica};
use crate::data::KINDS;
use crate::data::wire::Thing;
use crate::queries::model;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn spawn_thing(commands: &mut Commands, thing: Thing) -> Option<Entity> {
    let name = KINDS.get(thing.kind as usize)?;
    Some(commands.spawn((
        Name(format!("{name} {}", thing.id)),
        LocalTransform {
            translation: Vec3::new(thing.position[0], thing.position[1], thing.position[2]),
            rotation: quat_angle_axis(thing.facing, &Vec3::y()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Look(model(name)),
        Fit(theme::MARK_SCALE),
        Avatar(None),
        Replica(thing.id),
    )))
}
