use crate::components::{
    Agent, Clip, Facing, FoxName, GroundSpeed, NameColor, Nameplate, StepBack, StillFrames,
};
use crate::data::{FIRST_NAMES, FOX_MODEL, FOX_SCALE, LAST_NAMES};
use crate::queries;
use crate::resources::{Behavior, FoxModel, Names, Scene};
use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationClip, AnimationProperty};
use nightshade_graphics::commands::builtins::{despawn_entities_now, despawn_recursive_now};
use nightshade_text::contract::TextProperties;
use nightshade_text::prelude::*;
use rand::Rng;

pub fn ask_for_the_model(scene: &mut Scene<'_, '_>, model: &mut FoxModel) {
    model.asked = Some(nightshade_load::load(
        &mut *scene.stage.loading,
        nightshade_load::Source::Bytes(FOX_MODEL.to_vec()),
    ));
}

pub fn receive_the_model(
    scene: &mut Scene<'_, '_>,
    model: &mut FoxModel,
    outcome: Result<nightshade_prefab::data::ModelImport, String>,
) {
    let mut result = match outcome {
        Ok(result) => result,
        Err(reason) => {
            tracing::error!("Failed to load fox model: {}", reason);
            return;
        }
    };

    for (index, clip) in result.animations.iter().enumerate() {
        let lowered = clip.name.to_lowercase();
        if lowered.contains("survey") {
            model.clips.survey = Some(index);
        } else if lowered.contains("walk") {
            model.clips.walk = Some(index);
        } else if lowered.contains("run") {
            model.clips.run = Some(index);
        }
    }

    let root_bone_indices: std::collections::HashSet<usize> = [0, 1, 2, 3].into();
    model.filtered = result
        .animations
        .iter()
        .map(|clip| AnimationClip {
            name: clip.name.clone(),
            duration: clip.duration,
            channels: clip
                .channels
                .iter()
                .filter(|channel| {
                    if channel.target_property == AnimationProperty::Translation {
                        return false;
                    }
                    !(root_bone_indices.contains(&channel.target_node)
                        && channel.target_property == AnimationProperty::Rotation)
                })
                .cloned()
                .collect(),
            events: clip.events.clone(),
            source: 0,
        })
        .collect();

    model.skins = result.skins.clone();
    model.prefab = result.prefabs.first().cloned();

    nightshade_prefab::queue_model_load(&mut scene.prefab_spawn(), &mut result);

    model.loaded = true;
}

fn pick_a_name(names: &mut Names) -> (String, Vec4) {
    let max_combinations = FIRST_NAMES.len() * LAST_NAMES.len();
    if names.0.len() >= max_combinations {
        names.0.clear();
    }

    let mut generator = rand::rng();
    loop {
        let first_index = generator.random_range(0..FIRST_NAMES.len());
        let last_index = generator.random_range(0..LAST_NAMES.len());
        let full_name = format!("{} {}", FIRST_NAMES[first_index], LAST_NAMES[last_index]);

        if !names.0.contains(&full_name) {
            names.0.insert(full_name.clone());
            let color = queries::name_to_color(&full_name);
            return (full_name, color);
        }
    }
}

pub fn spawn_fox(
    scene: &mut Scene<'_, '_>,
    model: &FoxModel,
    names: &mut Names,
    behavior: &Behavior,
    position: Vec3,
) {
    if !model.loaded {
        return;
    }

    let Some(prefab) = &model.prefab else {
        return;
    };

    let agent_entity = scene.storage.spawn_entities::<(
        nightshade_scene::contract::LocalTransform,
        nightshade_scene::contract::GlobalTransform,
    )>(1)[0];

    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(agent_entity)
    {
        transform.translation = position;
    }

    scene.storage.set(
        agent_entity,
        nightshade_navmesh::contract::NavMeshAgent::new().with_speed(behavior.speed),
    );

    let fox_entity = nightshade_prefab::spawn_prefab_with_skins(
        &mut scene.prefab_spawn(),
        prefab,
        &model.filtered,
        &model.skins,
        Vec3::zeros(),
    );

    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(fox_entity)
    {
        transform.translation = position;
        transform.scale = Vec3::new(FOX_SCALE, FOX_SCALE, FOX_SCALE);
    }

    let initial_animation = model.clips.survey;
    if let Some(player) = scene
        .storage
        .get_mut::<nightshade_animation::contract::AnimationPlayer>(fox_entity)
    {
        if let Some(survey_index) = initial_animation {
            player.play(survey_index);
            player.speed = 0.5;
        } else if !player.clips.is_empty() {
            player.play(0);
            player.speed = 0.5;
        }
    }

    let (full_name, name_color) = pick_a_name(names);
    let name_position = position + nalgebra_glm::vec3(0.0, 1.5, 0.0);
    let name_entity = spawn_3d_billboard_text_with_properties(
        scene.stage.text_state,
        scene.commands,
        &full_name,
        name_position,
        TextProperties {
            font_size: 24.0,
            color: name_color,
            outline_width: 1.5,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );

    scene.storage.set(fox_entity, Agent(agent_entity));
    scene.storage.set(fox_entity, Nameplate(name_entity));
    scene.storage.set(fox_entity, FoxName(full_name));
    scene.storage.set(fox_entity, NameColor(name_color));
    scene.storage.set(fox_entity, Facing(0.0));
    scene.storage.set(fox_entity, Clip(initial_animation));
    scene.storage.set(fox_entity, GroundSpeed(0.0));
    scene.storage.set(fox_entity, StepBack(None));
    scene.storage.set(fox_entity, StillFrames(0));
}

pub fn clear_foxes(scene: &mut Scene<'_, '_>, names: &mut Names) {
    let foxes: Vec<(Entity, Entity, Entity)> = scene
        .storage
        .query_ref::<(&Agent, &Nameplate)>()
        .iter()
        .map(|(entity, (agent, nameplate))| (entity, agent.0, nameplate.0))
        .collect();

    for (fox, agent, nameplate) in foxes {
        let Scene { storage, stage, .. } = scene;
        let mut caches = nightshade_graphics::commands::builtins::EntityCaches {
            entities: stage.entities,
            meshes: stage.meshes,
            mesh_state: stage.mesh_state,
            textures: stage.textures,
            materials: stage.materials,
            hierarchy: stage.hierarchy,
        };
        despawn_entities_now(storage, &mut caches, &[agent, nameplate]);
        despawn_recursive_now(storage, &mut caches, fox);
    }

    names.0.clear();
}
