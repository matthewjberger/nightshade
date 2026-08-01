use crate::commands::fox;
use crate::components::{Agent, FoxName, NameColor};
use crate::queries;
use crate::resources::{Behavior, Ground, Interface, Names, Scene, Spawns, Stage, Trees};
use nightshade::prelude::*;
use nightshade_navmesh::commands::set_navmesh_debug_draw;
use nightshade_navmesh::resources::PathfindingAlgorithm;
use nightshade_platform::prelude::*;
use nightshade_ui::data::state::UiBase;
use nightshade_ui::prelude::*;
use rand::Rng;

pub fn poll_ui_events(
    mut behavior: ResMut<Behavior>,
    mut names: ResMut<Names>,
    mut spawns: ResMut<Spawns>,
    panel: Res<Interface>,
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

    let mut spawn_clicked = false;
    let mut clear_clicked = false;
    let mut algorithm_change: Option<usize> = None;
    let mut wander_change: Option<bool> = None;
    let mut follow_change: Option<bool> = None;

    for event in ui_events(scene.stage.frame) {
        match event {
            UiEvent::ButtonClicked(entity) => {
                if *entity == panel.spawn_button {
                    spawn_clicked = true;
                } else if *entity == panel.clear_button {
                    clear_clicked = true;
                }
            }
            UiEvent::DropdownChanged {
                entity,
                selected_index,
            } if *entity == panel.algorithm => {
                algorithm_change = Some(*selected_index);
            }
            UiEvent::CheckboxChanged { entity, value } => {
                if *entity == panel.wander_mode {
                    wander_change = Some(*value);
                } else if *entity == panel.follow_mode {
                    follow_change = Some(*value);
                }
            }
            _ => {}
        }
    }

    if let Some(value) = ui_checkbox_value(scene.storage, panel.show_navmesh)
        && value != behavior.show_navmesh
    {
        behavior.show_navmesh = value;
        set_navmesh_debug_draw(&mut *scene.stage.navmesh, value);
    }

    if let Some(value) = wander_change {
        behavior.wander = value;
    }

    if let Some(value) = follow_change {
        behavior.follow = value;
        let foxes: Vec<Entity> = scene
            .storage
            .query_ref::<(&Agent,)>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        behavior.followed =
            (value && !foxes.is_empty()).then(|| foxes[rand::rng().random_range(0..foxes.len())]);
        behavior.last_followed = None;
    }

    if let Some(value) = ui_slider_value(scene.storage, panel.speed)
        && (value - behavior.speed).abs() > f32::EPSILON
    {
        behavior.speed = value;
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
                agent.movement_speed = value;
            }
        }
    }

    if let Some(index) = algorithm_change
        && let Some(algorithm) = PathfindingAlgorithm::all().get(index)
    {
        scene.stage.navmesh.algorithm = *algorithm;
    }

    if spawn_clicked {
        let x = (rand::random::<f32>() - 0.5) * 30.0;
        let z = (rand::random::<f32>() - 0.5) * 30.0;
        let y = queries::sample_terrain_height(x, z, &ground.0);
        spawns.0.push(nalgebra_glm::vec3(x, y, z));
        behavior.last_followed = None;
    }

    if clear_clicked {
        fox::clear_foxes(scene, &mut names);
        behavior.followed = None;
        behavior.follow = false;
        if let Some(data) = scene
            .storage
            .get_mut::<nightshade_ui::components::UiCheckboxData>(panel.follow_mode)
        {
            data.value = false;
        }
        behavior.last_followed = None;
    }
}

pub fn update_ui_labels(
    mut behavior: ResMut<Behavior>,
    panel: Res<Interface>,
    trees: Res<Trees>,
    time: Res<Time>,
    navmesh_world: Res<nightshade_navmesh::resources::NavMeshWorld>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let fox_count = storage.query_ref::<(&Agent,)>().iter().count();
    let triangle_count = navmesh_world.triangles.len();
    let frames_per_second = time.frames_per_second;
    let tree_count = trees.count;

    let following = (behavior.followed != behavior.last_followed).then(|| {
        behavior.last_followed = behavior.followed;
        behavior.followed.and_then(|fox| {
            let name = storage.get::<FoxName>(fox)?.0.clone();
            let color = storage.get::<NameColor>(fox)?.0;
            Some((name, color))
        })
    });

    ui_set_text(
        &mut commands,
        panel.foxes_label,
        &format!("Foxes: {fox_count}"),
    );
    ui_set_text(
        &mut commands,
        panel.trees_label,
        &format!("Trees: {tree_count}"),
    );
    ui_set_text(
        &mut commands,
        panel.triangles_label,
        &format!("NavMesh Triangles: {triangle_count}"),
    );
    ui_set_text(
        &mut commands,
        panel.fps_label,
        &format!("FPS: {frames_per_second:.1}"),
    );

    let Some(following) = following else {
        return;
    };
    match following {
        Some((full_name, name_color)) => {
            ui_set_text(
                &mut commands,
                panel.following_label,
                &format!("Following: {full_name}"),
            );
            let label = panel.following_label;
            commands.queue(move |storage, _| {
                if let Some(node_color) =
                    storage.get_mut::<nightshade_ui::components::UiNodeColor>(label)
                {
                    node_color.colors[UiBase::INDEX] = Some(name_color);
                }
            });
        }
        None => ui_set_text(&mut commands, panel.following_label, ""),
    }
}
