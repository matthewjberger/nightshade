use crate::data;
use crate::resources::{Machine, Script, Store};
use nightshade::prelude::*;
use nightshade_navmesh::commands::generation::{RecastNavMeshConfig, generate_navmesh_recast};
use nightshade_navmesh::contract::NavMeshAgent;
use nightshade_navmesh::resources::NavMeshWorld;

fn plate(found: &mut (Vec<[f32; 3]>, Vec<[u32; 3]>), middle: Vec3, half: Vec3) {
    let start = found.0.len() as u32;
    let top = middle.y + half.y;
    found.0.push([middle.x - half.x, top, middle.z - half.z]);
    found.0.push([middle.x + half.x, top, middle.z - half.z]);
    found.0.push([middle.x + half.x, top, middle.z + half.z]);
    found.0.push([middle.x - half.x, top, middle.z + half.z]);
    found.1.push([start, start + 2, start + 1]);
    found.1.push([start, start + 3, start + 2]);
}

fn walls(found: &mut (Vec<[f32; 3]>, Vec<[u32; 3]>), middle: Vec3, half: Vec3) {
    let start = found.0.len() as u32;
    let low = middle.y - half.y;
    let high = middle.y + half.y;
    for (along, across) in [
        (-half.x, -half.z),
        (half.x, -half.z),
        (half.x, half.z),
        (-half.x, half.z),
    ] {
        found.0.push([middle.x + along, low, middle.z + across]);
        found.0.push([middle.x + along, high, middle.z + across]);
    }
    for corner in 0..4u32 {
        let next = (corner + 1) % 4;
        let one = start + corner * 2;
        let other = start + next * 2;
        found.1.push([one, other, one + 1]);
        found.1.push([other, other + 1, one + 1]);
    }
    plate(found, middle, half);
}

pub fn bake(
    script: Res<Script>,
    machine: Res<Machine>,
    mut store: ResMut<Store>,
    mut world: ResMut<NavMeshWorld>,
) {
    let known = &script.plan.known;
    let (Some(ground), Some(transform)) = (known.ground, known.transform) else {
        return;
    };
    if store.walked == store.shape {
        return;
    }
    store.walked = store.shape;
    let mut found: (Vec<[f32; 3]>, Vec<[u32; 3]>) = (Vec::new(), Vec::new());
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) || !store.holds(transform, row) {
            continue;
        }
        let place = store.read(transform, row);
        let middle = Vec3::new(place.axis(0), place.axis(1), place.axis(2));
        let size = match known.scale.filter(|key| store.holds(*key, row)) {
            Some(key) => {
                let held = store.read(key, row);
                Vec3::new(held.axis(0) * 0.5, held.axis(1) * 0.5, held.axis(2) * 0.5)
            }
            None => Vec3::new(0.5, 0.5, 0.5),
        };
        if store.holds(ground, row) {
            plate(&mut found, middle, size);
            continue;
        }
        let solid = known
            .body
            .filter(|key| store.holds(*key, row))
            .map(|key| crate::queries::show(&store, &machine, store.read(key, row)));
        if solid.is_some() {
            walls(&mut found, middle, size);
        }
    }
    if found.0.is_empty() {
        return;
    }
    let config = RecastNavMeshConfig {
        agent_radius: data::AGENT_RADIUS,
        agent_height: data::AGENT_HEIGHT,
        ..Default::default()
    };
    if let Some(made) = generate_navmesh_recast(&found.0, &found.1, &config) {
        *world = made;
    }
}

pub fn walk(
    script: Res<Script>,
    store: Res<Store>,
    mut commands: Commands,
    mut agents: Query<(&mut NavMeshAgent,)>,
) {
    let known = &script.plan.known;
    let Some(path) = known.path else {
        return;
    };
    let pace = known.pace;
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) || !store.holds(path, row) {
            continue;
        }
        let Some(entity) = store.entity(row) else {
            continue;
        };
        let held = store.read(path, row);
        let goal = Vec3::new(held.axis(0), held.axis(1), held.axis(2));
        let speed = pace
            .filter(|key| store.holds(*key, row))
            .map(|key| store.read(key, row).value())
            .unwrap_or(data::AGENT_PACE);
        let mut known_agent = false;
        agents.for_entity(entity, |(agent,)| {
            known_agent = true;
            agent.movement_speed = speed;
            let wanted = agent.target_position.unwrap_or(Vec3::zeros());
            if agent.target_position.is_none()
                || nalgebra_glm::distance(&wanted, &goal) > data::AGENT_STEP
            {
                agent.set_destination(goal);
            }
        });
        if !known_agent {
            let mut made = NavMeshAgent::new().with_speed(speed);
            made.set_destination(goal);
            commands.set(entity, made);
            continue;
        }
    }
}
