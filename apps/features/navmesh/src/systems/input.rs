use crate::components::Agent;
use crate::queries;
use crate::resources::{Behavior, FoxModel, Ground, Names, Scene, Stage};
use nightshade::prelude::*;
use nightshade_navmesh::contract::NavMeshAgentState;
use nightshade_platform::prelude::*;

pub fn handle_mouse_input(
    mut behavior: ResMut<Behavior>,
    mut names: ResMut<Names>,
    model: Res<FoxModel>,
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

    if pointer_captured(&*scene.stage.input_capture) {
        return;
    }

    let (left_clicked, right_clicked, mouse_position) = {
        let mouse = &scene.stage.input.mouse;
        (
            mouse.state.contains(MouseState::LEFT_JUST_PRESSED),
            mouse.state.contains(MouseState::RIGHT_JUST_PRESSED),
            mouse.position,
        )
    };

    if !left_clicked && !right_clicked {
        return;
    }

    let ground_position = nightshade_picking::screen_ray(
        &*scene.stage.window,
        scene.stage.viewport,
        &*scene.stage.active_camera,
        scene.storage.query_ref(),
        mouse_position,
    )
    .and_then(|ray| ray.intersect_ground_plane(0.0));

    let Some(ground_position) = ground_position else {
        return;
    };

    if ground_position.x.abs() > 35.0 || ground_position.z.abs() > 35.0 {
        return;
    }

    let terrain_y = queries::sample_terrain_height(ground_position.x, ground_position.z, &ground.0);
    let target = nalgebra_glm::vec3(ground_position.x, terrain_y, ground_position.z);

    if left_clicked {
        let agents: Vec<Entity> = scene
            .storage
            .query_ref::<(&Agent,)>()
            .iter()
            .map(|(_entity, (agent,))| agent.0)
            .collect();
        for agent_entity in agents {
            if let Some(agent) = scene
                .storage
                .get_mut::<nightshade_navmesh::contract::NavMeshAgent>(agent_entity)
            {
                agent.target_position = Some(target);
                agent.state = NavMeshAgentState::PathPending;
            }
        }
        return;
    }

    crate::commands::fox::spawn_fox(scene, &model, &mut names, &behavior, target);
    behavior.last_followed = None;
}
