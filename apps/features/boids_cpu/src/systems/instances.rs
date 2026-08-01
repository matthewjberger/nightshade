use crate::components::Shade;
use crate::data::{Applied, fill_by_index};
use crate::resources::{BoidsResources, GridBuilder, Integration};
use nalgebra_glm::{Mat3, Quat, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::components::mesh::InstanceCustomData;

pub const BOID_SCALE: Vec3 = Vec3::new(1.6, 5.0, 1.6);

pub fn step_flock(world: &mut World) {
    let delta_time = world.res::<Time>().delta_time;
    if delta_time <= 0.0 {
        return;
    }

    world.resource_scope::<BoidsResources, ()>(|ecs, resources| {
        let member = ecs;
        if let Some(integration) = member.resources.get_mut::<Integration>() {
            integration.delta_time = delta_time.min(0.05);
        }

        let started = Instant::now();
        resources
            .flock
            .get_mut()
            .expect("the flock schedule lock is never held across a frame")
            .run(member);
        resources.step_seconds += started.elapsed().as_secs_f64();
        resources.step_count += 1;
    });
}

pub fn sync_instances(
    mut resources: ResMut<BoidsResources>,
    applied: Option<Res<Applied>>,
    integration: Option<Res<Integration>>,
    grid: Option<Res<GridBuilder>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(draw) = resources.draw else {
        return;
    };

    let visible = resources.show_boids;
    let mut buffer = std::mem::take(&mut resources.instances);
    let started = Instant::now();

    let wanted = if visible {
        applied
            .as_ref()
            .map(|applied| applied.count())
            .unwrap_or_default()
    } else {
        0
    };

    buffer.clear();
    buffer.resize(wanted, InstanceTransform::default());

    if wanted > 0
        && let Some(applied) = applied.as_ref()
    {
        let positions = &applied.position[..wanted];
        let rotations = &applied.rotation[..wanted];

        fill_by_index(&mut buffer, |index| {
            InstanceTransform::new(positions[index], rotations[index], BOID_SCALE)
        });
    }

    if let Some(mesh) = storage.get_mut::<InstancedMesh>(draw) {
        std::mem::swap(&mut mesh.instances, &mut buffer);
        mesh.mark_instances_changed();
    }

    let by_region = resources.show_bounds;
    let stale = storage
        .get::<InstancedMesh>(draw)
        .is_some_and(|mesh| mesh.custom_data.len() != mesh.instances.len());

    if by_region {
        tint_by_region(&mut resources, integration, grid, storage, draw);
    } else if stale || resources.tinted_by_region {
        recolor_instances(storage, draw);
    }
    resources.tinted_by_region = by_region;

    resources.instances = buffer;
    resources.sync_seconds += started.elapsed().as_secs_f64();
}

fn tint_by_region(
    resources: &mut BoidsResources,
    integration: Option<Res<Integration>>,
    grid: Option<Res<GridBuilder>>,
    storage: &mut nightshade::ecs::Storage,
    draw: Entity,
) {
    let Some(regions) = integration.map(|integration| integration.regions) else {
        return;
    };
    let Some(builder) = grid else {
        return;
    };

    let entries = builder.grid.entries.clone();
    let count = entries.len();
    if count == 0 || regions == 0 {
        return;
    }

    let mut tints = std::mem::take(&mut resources.tints);
    tints.clear();
    tints.resize(count, InstanceCustomData::default());

    for (slot, &gathered) in entries.iter().enumerate() {
        tints[gathered as usize].tint =
            crate::systems::population::shade_for(slot * regions / count);
    }

    if let Some(mesh) = storage.get_mut::<InstancedMesh>(draw) {
        tints.resize(mesh.instances.len(), InstanceCustomData::default());
        std::mem::swap(&mut mesh.custom_data, &mut tints);
    }
    resources.tints = tints;
}

fn recolor_instances(storage: &mut nightshade::ecs::Storage, draw: Entity) {
    let mut tints: Vec<InstanceCustomData> = Vec::new();
    for (_entity, (shade,)) in storage.query_ref::<(&Shade,)>().iter() {
        tints.push(InstanceCustomData { tint: shade.0 });
    }

    if let Some(mesh) = storage.get_mut::<InstancedMesh>(draw) {
        tints.resize(mesh.instances.len(), InstanceCustomData::default());
        mesh.custom_data = tints;
    }
}

pub fn heading_rotation(velocity: Vec3) -> Quat {
    let speed = nalgebra_glm::length(&velocity);
    if speed < f32::EPSILON {
        return Quat::identity();
    }

    let forward = velocity / speed;
    let reference = if forward.y.abs() > 0.99 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&reference, &forward));
    let up = nalgebra_glm::cross(&right, &forward);

    nalgebra_glm::mat3_to_quat(&Mat3::from_columns(&[right, forward, up])).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_still_boid_keeps_its_orientation() {
        assert_eq!(heading_rotation(Vec3::zeros()), Quat::identity());
    }

    #[test]
    fn the_cone_axis_ends_up_along_the_heading() {
        let heading = nalgebra_glm::normalize(&Vec3::new(1.0, 0.0, 2.0));

        let pointed = nalgebra_glm::quat_rotate_vec3(
            &heading_rotation(heading * 5.0),
            &Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(nalgebra_glm::length(&(pointed - heading)) < 1.0e-4);
    }

    #[test]
    fn a_boid_flying_straight_up_still_resolves() {
        let heading = Vec3::new(0.0, 1.0, 0.0);

        let pointed = nalgebra_glm::quat_rotate_vec3(
            &heading_rotation(heading * 3.0),
            &Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(nalgebra_glm::length(&(pointed - heading)) < 1.0e-4);
    }
}
