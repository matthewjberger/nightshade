use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_scene::prelude::*;

use crate::commands::{blend_space, jump_arc, rig};
use crate::data;
use crate::queries::clips::missing;
use crate::resources::LocomotionResources;

pub fn attach_the_model(
    mut locomotion_resources: ResMut<LocomotionResources>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = locomotion_resources.character.asked else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut *loading, handle) else {
        return;
    };
    locomotion_resources.character.asked = None;

    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("the character did not open: {error}");
            return;
        }
    };
    nightshade_prefab::queue_model_load(
        &mut nightshade_prefab::PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        },
        &mut import,
    );
    let Some(prefab) = import.prefabs.first().cloned() else {
        return;
    };
    let model = nightshade_prefab::spawn_prefab_with_skins(
        &mut nightshade_prefab::PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        },
        &prefab,
        &import.animations,
        &import.skins,
        Vec3::zeros(),
    );

    if let Some(name) = storage.get_mut::<Name>(model) {
        name.0 = "Character Model".to_string();
    }
    if let Some(body) = locomotion_resources.character.body {
        storage.add_components::<(nightshade::ecs::ChildOf,)>(model);
        storage.set(model, ChildOf(body));
    }
    if let Some(transform) = storage.get_mut::<LocalTransform>(model) {
        transform.translation = Vec3::zeros();
    }
    if let Some(camera) = locomotion_resources.character.camera
        && let Some(follow) =
            storage.get_mut::<nightshade_camera_controller::contract::ThirdPersonCamera>(camera)
    {
        follow.height_offset = crate::data::CAMERA_HEIGHT;
    }

    let held = locomotion_resources.character.held_clips.clone();

    let joints = nightshade_animation::skeleton_joints(storage, parts.hierarchy, model);
    let bones: Vec<String> = joints.iter().map(|(name, _)| name.clone()).collect();

    if storage.get::<AnimationPlayer>(model).is_none() {
        let player = AnimationPlayer {
            bone_name_to_entity: joints.iter().cloned().collect(),
            resolve_by_bone_name: true,
            ..Default::default()
        };
        storage.add_components::<(AnimationPlayer,)>(model);
        storage.set(model, player);
    }

    if let Some(player) = storage.get_mut::<AnimationPlayer>(model) {
        match held {
            Some(set) => {
                let report = nightshade_animation::clip_set_apply(player, &bones, &set);
                if !report.is_complete() {
                    tracing::error!(
                        "{} channels found no joint on this skeleton, missing {}",
                        report.channels_dropped,
                        report.unresolved_bones.join(", ")
                    );
                }
            }
            None => {
                let absent = missing(
                    player,
                    &[
                        data::IDLE,
                        data::WALKING,
                        data::RUNNING,
                        data::LEFT_STRAFE,
                        data::RIGHT_STRAFE,
                        data::LEFT_STRAFE_WALKING,
                        data::RIGHT_STRAFE_WALKING,
                    ],
                );
                if absent.is_empty() {
                    rig::drop_bone_offsets(player);
                    blend_space::hold_the_root_still(player);
                    jump_arc::raise_the_jump(player);
                    for name in [
                        data::JUMP,
                        data::LEFT_TURN,
                        data::RIGHT_TURN,
                        data::LEFT_TURN_WIDE,
                        data::RIGHT_TURN_WIDE,
                    ] {
                        rig::hold_the_facing(player, name);
                    }
                } else {
                    tracing::error!(
                        "the character is missing these clips: {}",
                        absent.join(", ")
                    );
                }
            }
        }
        blend_space::build(player);
    }

    let character = &mut locomotion_resources.character;
    character.model = Some(model);
    character.swapping = false;
}
