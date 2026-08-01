use crate::commands::fox;
use crate::components::{Agent, Clip, Facing, GroundSpeed, Nameplate, StepBack, StillFrames};
use crate::data::{
    IDLE_SPEED_THRESHOLD, INITIAL_FOXES, PLAUSIBLE_STEP_MULTIPLE, RUN_CLIP_GROUND_SPEED,
    RUN_DISTANCE_THRESHOLD, STILL_FRAMES_FOR_IDLE, WALK_CLIP_GROUND_SPEED,
};
use crate::queries;
use crate::resources::{Behavior, FoxModel, Ground, Names, Scene, Spawns, Stage};
use nightshade::prelude::*;
use nightshade_navmesh::contract::NavMeshAgentState;
use nightshade_platform::prelude::*;

pub fn stand_the_foxes(
    mut spawns: ResMut<Spawns>,
    mut model: ResMut<FoxModel>,
    mut names: ResMut<Names>,
    behavior: Res<Behavior>,
    ground: Res<Ground>,
    stage: Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut owned = Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    let scene = &mut owned;

    for position in std::mem::take(&mut spawns.0) {
        fox::spawn_fox(scene, &model, &mut names, &behavior, position);
    }

    let Some(handle) = model.asked else {
        return;
    };
    let Some(outcome) = nightshade_load::take(scene.stage.loading, handle) else {
        return;
    };
    model.asked = None;
    fox::receive_the_model(scene, &mut model, outcome);

    for (x, z) in INITIAL_FOXES {
        let y = queries::sample_terrain_height(x, z, &ground.0);
        fox::spawn_fox(
            scene,
            &model,
            &mut names,
            &behavior,
            nalgebra_glm::vec3(x, y, z),
        );
    }
}

pub fn sync_foxes_to_agents(
    time: Res<Time>,
    ground: Res<Ground>,
    behavior: Res<Behavior>,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = time.delta_time;
    let agent_speed = behavior.speed;

    let foxes: Vec<(Entity, Entity, Entity)> = storage
        .query_ref::<(&Agent, &Nameplate)>()
        .iter()
        .map(|(entity, (agent, nameplate))| (entity, agent.0, nameplate.0))
        .collect();

    for (fox, agent_entity, name_entity) in foxes {
        let agent_position = storage
            .get::<nightshade_scene::contract::LocalTransform>(agent_entity)
            .map(|transform| transform.translation)
            .unwrap_or(Vec3::zeros());

        let terrain_y =
            queries::sample_terrain_height(agent_position.x, agent_position.z, &ground.0);
        let fox_position = Vec3::new(agent_position.x, terrain_y, agent_position.z);

        let mut facing = storage
            .get::<Facing>(fox)
            .map(|value| value.0)
            .unwrap_or(0.0);
        let agent = storage.get::<nightshade_navmesh::contract::NavMeshAgent>(agent_entity);
        let moving = agent
            .map(|state| state.state == NavMeshAgentState::Moving)
            .unwrap_or(false);

        if moving
            && let Some(agent) = agent
            && let Some(waypoint) = agent.current_waypoint()
        {
            let direction = waypoint - agent_position;
            if direction.x.abs() > 0.01 || direction.z.abs() > 0.01 {
                let target_rotation = direction.x.atan2(direction.z);
                let rotation_speed = 10.0;

                let mut angle_difference = target_rotation - facing;
                while angle_difference > std::f32::consts::PI {
                    angle_difference -= 2.0 * std::f32::consts::PI;
                }
                while angle_difference < -std::f32::consts::PI {
                    angle_difference += 2.0 * std::f32::consts::PI;
                }
                facing += angle_difference * rotation_speed * delta_time;
            }
        }

        let previous = storage.get::<StepBack>(fox).and_then(|value| value.0);
        let held_speed = storage
            .get::<GroundSpeed>(fox)
            .map(|value| value.0)
            .unwrap_or(0.0);
        let measured_speed = match previous {
            Some(previous) if delta_time > 0.0 => {
                let travelled =
                    nalgebra_glm::vec2(fox_position.x - previous.x, fox_position.z - previous.z)
                        .norm();
                if travelled > agent_speed * PLAUSIBLE_STEP_MULTIPLE * delta_time {
                    held_speed
                } else {
                    travelled / delta_time
                }
            }
            _ => 0.0,
        };

        let still_frames = storage
            .get::<StillFrames>(fox)
            .map(|value| value.0)
            .unwrap_or(0);
        let still_frames = if measured_speed < IDLE_SPEED_THRESHOLD {
            still_frames.saturating_add(1)
        } else {
            0
        };

        storage.set(fox, Facing(facing));
        storage.set(fox, GroundSpeed(measured_speed));
        storage.set(fox, StillFrames(still_frames));
        storage.set(fox, StepBack(Some(fox_position)));

        if let Some(transform) = storage.get_mut::<nightshade_scene::contract::LocalTransform>(fox)
        {
            transform.translation = fox_position;
            transform.rotation = nalgebra_glm::quat_angle_axis(facing, &Vec3::y());
        }

        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(name_entity)
        {
            transform.translation = fox_position + nalgebra_glm::vec3(0.0, 1.5, 0.0);
        }
    }
}

pub fn update_fox_animations(model: Res<FoxModel>, storage: &mut nightshade::ecs::Storage) {
    let foxes: Vec<(Entity, Entity)> = storage
        .query_ref::<(&Agent,)>()
        .iter()
        .map(|(entity, (agent,))| (entity, agent.0))
        .collect();

    for (fox, agent_entity) in foxes {
        let distance = storage
            .get::<nightshade_navmesh::contract::NavMeshAgent>(agent_entity)
            .map(|agent| agent.distance_to_destination)
            .unwrap_or(0.0);

        let still_frames = storage
            .get::<StillFrames>(fox)
            .map(|value| value.0)
            .unwrap_or(0);
        let ground_speed = storage
            .get::<GroundSpeed>(fox)
            .map(|value| value.0)
            .unwrap_or(0.0);
        let current = storage.get::<Clip>(fox).and_then(|value| value.0);

        let (target_animation, target_speed) = if still_frames >= STILL_FRAMES_FOR_IDLE {
            (model.clips.survey, 0.5)
        } else if distance > RUN_DISTANCE_THRESHOLD {
            (model.clips.run, ground_speed / RUN_CLIP_GROUND_SPEED)
        } else {
            (model.clips.walk, ground_speed / WALK_CLIP_GROUND_SPEED)
        };

        if target_animation != current {
            if let Some(animation_index) = target_animation
                && let Some(player) =
                    storage.get_mut::<nightshade_animation::contract::AnimationPlayer>(fox)
            {
                player.blend_to_in_phase(animation_index, 0.3);
                player.speed = target_speed;
                storage.set(fox, Clip(Some(animation_index)));
            }
        } else if let Some(player) =
            storage.get_mut::<nightshade_animation::contract::AnimationPlayer>(fox)
        {
            player.speed = target_speed;
        }
    }
}
