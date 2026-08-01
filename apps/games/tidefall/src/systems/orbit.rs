use crate::components::{Around, Motion, Offset, Pull, SpinRate, Velocity};
use crate::data::{MAX_STEP, Settings};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use std::collections::HashMap;

const DEPTH: usize = 4;

pub fn circle(
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut bodies: Query<(
        &Around,
        &mut Offset,
        &mut Velocity,
        &SpinRate,
        &Pull,
        &mut Motion,
        &mut LocalTransform,
    )>,
) {
    let real = time.delta_time.min(MAX_STEP);
    let delta_time = real * config.number(Settings::Clock);
    if delta_time <= f32::EPSILON {
        bodies.for_each_mut(|_entity, (_, _, _, _, _, motion, _)| motion.0 = Vec3::zeros());
        return;
    }

    let mut paths = Vec::new();
    bodies.for_each_mut(|entity, (around, offset, velocity, _, pull, _, _)| {
        paths.push((entity, around.0, offset.0, velocity.0, pull.0));
    });
    let pulls: HashMap<Entity, f32> = paths.iter().map(|path| (path.0, path.4)).collect();

    let mut centers: HashMap<Entity, Vec3> = HashMap::new();
    let mut resolved = vec![false; paths.len()];
    for _ in 0..DEPTH {
        for (index, path) in paths.iter_mut().enumerate() {
            if resolved[index] {
                continue;
            }
            let (entity, around, offset, velocity, _) = path;
            let Some(primary) = around else {
                centers.insert(*entity, *offset);
                resolved[index] = true;
                continue;
            };
            let Some(primary_center) = centers.get(primary).copied() else {
                continue;
            };
            let pull = pulls.get(primary).copied().unwrap_or(0.0);
            let distance = offset.magnitude();
            if distance > f32::EPSILON && pull > 0.0 {
                *velocity -= *offset / distance * (pull / distance) * delta_time;
            }
            *offset += *velocity * delta_time;
            centers.insert(*entity, primary_center + *offset);
            resolved[index] = true;
        }
    }

    for (index, (entity, _, offset, velocity, _)) in paths.iter().enumerate() {
        if !resolved[index] {
            continue;
        }
        let Some(center) = centers.get(entity).copied() else {
            continue;
        };
        bodies.for_entity(
            *entity,
            |(_, offset_component, velocity_component, spin, _, motion, transform)| {
                offset_component.0 = *offset;
                velocity_component.0 = *velocity;
                motion.0 = (center - transform.translation) / real;
                transform.translation = center;
                transform.rotation = nalgebra_glm::quat_normalize(
                    &(transform.rotation
                        * nalgebra_glm::quat_angle_axis(spin.0 * delta_time, &Vec3::y())),
                );
            },
        );
    }
}
