use crate::resources::{FootPlant, GaitTrack, IkChain, PoseKeyframe, SandboxResources, SoftJoint};
use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationClip, AnimationProperty, AnimationSamplerOutput};
use nightshade_scene::queries::query_children;
use std::collections::HashMap;

const POSE_BLEND_SHARPNESS: f32 = 12.0;
const GAIT_TRACKING_SHARPNESS_BOOST: f32 = 64.0;
const GAIT_PHASE_LEAD: f32 = 0.35;
const STRIDE_STANCE_WARP: f32 = 0.18;
const REACH_ARM_DEGREES: f32 = 90.0;
const IDLE_ARM_DROP_DEGREES: f32 = 79.0;
const IDLE_ELBOW_BEND_DEGREES: f32 = 11.0;
const IDLE_ARM_FORWARD_DEGREES: f32 = 4.0;
const RUN_ARM_CONVERGENCE: f32 = 12.0;
const WALK_ARM_CONVERGENCE: f32 = 7.0;
const CROUCH_HIPS_DROP: f32 = 0.22;
const CROUCH_MAX_WEIGHT: f32 = 1.15;
const LOOK_JOINTS: &[(&str, f32)] = &[
    ("Spine1", 0.12),
    ("Spine2", 0.18),
    ("Neck", 0.3),
    ("Head", 0.4),
];
const SOFT_JOINTS: &[(&str, f32, f32)] = &[
    ("LeftForeArm", 14000.0, 0.55),
    ("RightForeArm", 14000.0, 0.55),
    ("LeftArm", 15000.0, 0.55),
    ("RightArm", 15000.0, 0.55),
    ("Head", 9000.0, 0.6),
    ("Spine2", 18000.0, 0.6),
];
const SOFTNESS_SUBSTEP: f32 = 1.0 / 240.0;
const SOFTNESS_MAX_SUBSTEPS: usize = 16;
const SOFTNESS_MAX_STEP: f32 = 0.1;

const WALK_DUTY: f32 = 0.62;
const RUN_DUTY: f32 = 0.31;
const CROUCH_DUTY: f32 = 0.68;
const WALK_STANCE_TRAVEL_SLOW: f32 = 0.55;
const WALK_STANCE_TRAVEL: f32 = 0.9;
const RUN_STANCE_TRAVEL: f32 = 1.0;
const WALK_CONTACT_FRACTION: f32 = 0.42;
const RUN_CONTACT_FRACTION: f32 = 0.32;
const CROUCH_LEG_FRACTION: f32 = 0.68;
const MINIMUM_STRIDE_RADIUS: f32 = 0.05;
const FALLBACK_FOOT_LENGTH_FRACTION: f32 = 0.155;

const BREATH_HERTZ: f32 = 0.24;
const SWAY_HERTZ: f32 = 0.078;
const IDLE_CLOCK_WRAP: f32 = 1000.0;

const IDLE_LIFE: &[(&str, char, f32, f32)] = &[
    ("Spine", 'x', -1.6, 0.0),
    ("Spine1", 'x', -1.1, 0.0),
    ("Spine2", 'x', -0.7, 0.0),
    ("Hips", 'z', 0.0, 2.1),
    ("Spine", 'z', 0.0, -1.5),
    ("Neck", 'z', 0.0, 0.6),
    ("Head", 'y', 0.0, 1.8),
    ("LeftShoulder", 'z', 0.9, 0.0),
    ("RightShoulder", 'z', -0.9, 0.0),
    ("LeftArm", 'x', -0.5, 1.4),
    ("RightArm", 'x', -0.5, -1.4),
    ("LeftForeArm", 'x', -0.7, 0.0),
    ("RightForeArm", 'x', -0.7, 0.0),
];

const CROUCH: &[(&str, char, f32)] = &[
    ("Spine", 'x', 15.0),
    ("Spine1", 'x', 10.0),
    ("LeftArm", 'z', -65.0),
    ("RightArm", 'z', 65.0),
    ("LeftArm", 'x', -20.0),
    ("RightArm", 'x', -20.0),
    ("LeftForeArm", 'x', -30.0),
    ("RightForeArm", 'x', -30.0),
    ("LeftUpLeg", 'x', -55.0),
    ("RightUpLeg", 'x', -55.0),
    ("LeftLeg", 'x', 75.0),
    ("RightLeg", 'x', 75.0),
    ("LeftFoot", 'x', -18.0),
    ("RightFoot", 'x', -18.0),
];

const GAIT_REACH: &[(&str, char, f32)] = &[
    ("Hips", 'y', -12.0),
    ("LeftArm", 'z', -75.0),
    ("RightArm", 'z', 75.0),
    ("LeftArm", 'x', 30.0),
    ("RightArm", 'x', -30.0),
    ("LeftArm", 'y', -RUN_ARM_CONVERGENCE),
    ("RightArm", 'y', RUN_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -72.0),
    ("RightForeArm", 'x', -88.0),
    ("LeftShoulder", 'y', 8.0),
    ("RightShoulder", 'y', 8.0),
    ("Spine", 'x', 16.0),
    ("Spine1", 'x', 4.0),
    ("Spine", 'y', 11.0),
    ("Spine1", 'y', 5.0),
    ("Spine2", 'y', 3.0),
    ("Neck", 'x', -14.0),
    ("Neck", 'y', -6.0),
    ("Head", 'y', -2.0),
];

const GAIT_PASS: &[(&str, char, f32)] = &[
    ("Hips", 'y', -6.0),
    ("Hips", 'z', 4.0),
    ("LeftArm", 'z', -75.0),
    ("RightArm", 'z', 75.0),
    ("LeftArm", 'x', 15.0),
    ("RightArm", 'x', -15.0),
    ("LeftArm", 'y', -RUN_ARM_CONVERGENCE),
    ("RightArm", 'y', RUN_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -76.0),
    ("RightForeArm", 'x', -84.0),
    ("LeftShoulder", 'y', 4.0),
    ("RightShoulder", 'y', 4.0),
    ("Spine", 'x', 18.0),
    ("Spine1", 'x', 5.0),
    ("Spine", 'y', 6.0),
    ("Spine1", 'y', 3.0),
    ("Spine2", 'y', 1.0),
    ("Neck", 'x', -15.0),
    ("Neck", 'y', -3.0),
    ("Head", 'y', -1.0),
];

const GAIT_PUSH: &[(&str, char, f32)] = &[
    ("Hips", 'y', 6.0),
    ("Hips", 'z', 2.0),
    ("LeftArm", 'z', -75.0),
    ("RightArm", 'z', 75.0),
    ("LeftArm", 'x', -15.0),
    ("RightArm", 'x', 15.0),
    ("LeftArm", 'y', -RUN_ARM_CONVERGENCE),
    ("RightArm", 'y', RUN_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -84.0),
    ("RightForeArm", 'x', -76.0),
    ("LeftShoulder", 'y', -4.0),
    ("RightShoulder", 'y', -4.0),
    ("Spine", 'x', 20.0),
    ("Spine1", 'x', 6.0),
    ("Spine", 'y', -6.0),
    ("Spine1", 'y', -3.0),
    ("Spine2", 'y', -2.0),
    ("Neck", 'x', -16.0),
    ("Neck", 'y', 3.0),
    ("Head", 'y', 1.0),
];

const WALK_REACH: &[(&str, char, f32)] = &[
    ("Hips", 'y', -7.0),
    ("LeftArm", 'z', -78.0),
    ("RightArm", 'z', 78.0),
    ("LeftArm", 'x', 18.0),
    ("RightArm", 'x', -18.0),
    ("LeftArm", 'y', -WALK_ARM_CONVERGENCE),
    ("RightArm", 'y', WALK_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -22.0),
    ("RightForeArm", 'x', -34.0),
    ("LeftShoulder", 'y', 5.0),
    ("RightShoulder", 'y', 5.0),
    ("Spine", 'x', 2.0),
    ("Spine", 'y', 6.0),
    ("Neck", 'x', -3.0),
];

const WALK_PASS: &[(&str, char, f32)] = &[
    ("Hips", 'y', -3.0),
    ("Hips", 'z', 5.0),
    ("LeftArm", 'z', -78.0),
    ("RightArm", 'z', 78.0),
    ("LeftArm", 'x', 9.0),
    ("RightArm", 'x', -9.0),
    ("LeftArm", 'y', -WALK_ARM_CONVERGENCE),
    ("RightArm", 'y', WALK_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -25.0),
    ("RightForeArm", 'x', -31.0),
    ("LeftShoulder", 'y', 2.5),
    ("RightShoulder", 'y', 2.5),
    ("Spine", 'x', 3.0),
    ("Spine", 'y', 0.0),
    ("Neck", 'x', -3.0),
];

const WALK_PUSH: &[(&str, char, f32)] = &[
    ("Hips", 'y', 4.0),
    ("Hips", 'z', 3.0),
    ("LeftArm", 'z', -78.0),
    ("RightArm", 'z', 78.0),
    ("LeftArm", 'x', -9.0),
    ("RightArm", 'x', 9.0),
    ("LeftArm", 'y', -WALK_ARM_CONVERGENCE),
    ("RightArm", 'y', WALK_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -31.0),
    ("RightForeArm", 'x', -25.0),
    ("LeftShoulder", 'y', -2.5),
    ("RightShoulder", 'y', -2.5),
    ("Spine", 'x', 4.0),
    ("Spine", 'y', -6.0),
    ("Neck", 'x', -3.0),
    ("Neck", 'y', 3.0),
];

const CROUCH_WALK_REACH: &[(&str, char, f32)] = &[
    ("Hips", 'y', -6.0),
    ("LeftArm", 'z', -78.0),
    ("RightArm", 'z', 78.0),
    ("LeftArm", 'x', 15.0),
    ("RightArm", 'x', -15.0),
    ("LeftArm", 'y', -WALK_ARM_CONVERGENCE),
    ("RightArm", 'y', WALK_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -35.0),
    ("RightForeArm", 'x', -42.0),
    ("LeftShoulder", 'y', 4.0),
    ("RightShoulder", 'y', 4.0),
    ("Spine", 'x', 16.0),
    ("Spine", 'y', 5.0),
];

const CROUCH_WALK_PASS: &[(&str, char, f32)] = &[
    ("Hips", 'y', -3.0),
    ("Hips", 'z', 4.0),
    ("LeftArm", 'z', -78.0),
    ("RightArm", 'z', 78.0),
    ("LeftArm", 'x', 8.0),
    ("RightArm", 'x', -8.0),
    ("LeftArm", 'y', -WALK_ARM_CONVERGENCE),
    ("RightArm", 'y', WALK_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -37.0),
    ("RightForeArm", 'x', -41.0),
    ("LeftShoulder", 'y', 2.0),
    ("RightShoulder", 'y', 2.0),
    ("Spine", 'x', 16.0),
    ("Spine", 'y', 0.0),
];

const CROUCH_WALK_PUSH: &[(&str, char, f32)] = &[
    ("Hips", 'y', 3.0),
    ("Hips", 'z', 2.0),
    ("LeftArm", 'z', -78.0),
    ("RightArm", 'z', 78.0),
    ("LeftArm", 'x', -8.0),
    ("RightArm", 'x', 8.0),
    ("LeftArm", 'y', -WALK_ARM_CONVERGENCE),
    ("RightArm", 'y', WALK_ARM_CONVERGENCE),
    ("LeftForeArm", 'x', -41.0),
    ("RightForeArm", 'x', -37.0),
    ("LeftShoulder", 'y', -2.0),
    ("RightShoulder", 'y', -2.0),
    ("Spine", 'x', 16.0),
    ("Spine", 'y', -5.0),
];

const AIR_RISE: &[(&str, char, f32)] = &[
    ("LeftUpLeg", 'x', -45.0),
    ("RightUpLeg", 'x', -18.0),
    ("LeftLeg", 'x', 72.0),
    ("RightLeg", 'x', 45.0),
    ("LeftFoot", 'x', -12.0),
    ("RightFoot", 'x', -18.0),
    ("LeftArm", 'z', -60.0),
    ("RightArm", 'z', 60.0),
    ("LeftArm", 'x', -40.0),
    ("RightArm", 'x', -40.0),
    ("LeftForeArm", 'x', -70.0),
    ("RightForeArm", 'x', -70.0),
    ("Spine", 'x', 8.0),
];

const AIR_FALL: &[(&str, char, f32)] = &[
    ("LeftUpLeg", 'x', -15.0),
    ("RightUpLeg", 'x', -5.0),
    ("LeftLeg", 'x', 35.0),
    ("RightLeg", 'x', 30.0),
    ("LeftFoot", 'x', -15.0),
    ("RightFoot", 'x', -15.0),
    ("LeftArm", 'z', -55.0),
    ("RightArm", 'z', 55.0),
    ("LeftArm", 'x', 12.0),
    ("RightArm", 'x', 12.0),
    ("LeftForeArm", 'x', -30.0),
    ("RightForeArm", 'x', -30.0),
    ("Spine", 'x', -4.0),
];

const TUCK_FORWARD: &[(&str, char, f32)] = &[
    ("LeftUpLeg", 'x', -95.0),
    ("RightUpLeg", 'x', -90.0),
    ("LeftLeg", 'x', 120.0),
    ("RightLeg", 'x', 115.0),
    ("LeftFoot", 'x', -20.0),
    ("RightFoot", 'x', -20.0),
    ("LeftArm", 'z', -45.0),
    ("RightArm", 'z', 45.0),
    ("LeftArm", 'x', -60.0),
    ("RightArm", 'x', -60.0),
    ("LeftForeArm", 'x', -110.0),
    ("RightForeArm", 'x', -110.0),
    ("Spine", 'x', 30.0),
    ("Spine1", 'x', 15.0),
    ("Neck", 'x', 20.0),
];

const TUCK_SIDE: &[(&str, char, f32)] = &[
    ("LeftUpLeg", 'x', -95.0),
    ("RightUpLeg", 'x', -90.0),
    ("LeftLeg", 'x', 120.0),
    ("RightLeg", 'x', 115.0),
    ("LeftFoot", 'x', -20.0),
    ("RightFoot", 'x', -20.0),
    ("LeftArm", 'z', -45.0),
    ("RightArm", 'z', 45.0),
    ("LeftArm", 'x', -60.0),
    ("RightArm", 'x', -60.0),
    ("LeftForeArm", 'x', -110.0),
    ("RightForeArm", 'x', -110.0),
    ("Spine", 'x', 22.0),
    ("Spine", 'z', 15.0),
    ("Spine1", 'x', 10.0),
    ("Spine1", 'z', 10.0),
    ("Neck", 'x', 15.0),
];

const RELAXED_FINGER_CURL: f32 = 18.0;
const GRIP_FINGER_CURL: f32 = 35.0;
const FINGERS: &[&str] = &["Thumb", "Index", "Middle", "Ring", "Pinky"];

fn relaxed_finger_curl() -> Vec<(String, Vec3, f32)> {
    let mut deltas = Vec::new();
    for (side, sign) in [("Left", -1.0), ("Right", 1.0)] {
        for finger in FINGERS {
            let strength = if *finger == "Thumb" { 0.5 } else { 1.0 };
            for segment in 1..=3 {
                deltas.push((
                    format!("{side}Hand{finger}{segment}"),
                    Vec3::z(),
                    RELAXED_FINGER_CURL * sign * strength,
                ));
            }
        }
    }
    deltas
}

fn grip_finger_curl() -> Vec<(String, Vec3, f32)> {
    let mut deltas = Vec::new();
    for side in ["Left", "Right"] {
        for finger in FINGERS {
            let strength = if *finger == "Thumb" { 0.5 } else { 1.0 };
            for segment in 1..=3 {
                deltas.push((
                    format!("{side}Hand{finger}{segment}"),
                    Vec3::x(),
                    GRIP_FINGER_CURL * strength,
                ));
            }
        }
    }
    deltas
}

fn table_keyframe(
    scene: &crate::commands::Scene<'_, '_>,
    model: Entity,
    rest_rotations: &HashMap<Entity, nalgebra_glm::Quat>,
    bones: &HashMap<String, Entity>,
    table: &[(&str, char, f32)],
    finger_deltas: &[(String, Vec3, f32)],
) -> Vec<(Entity, nalgebra_glm::Quat)> {
    let mut deltas: Vec<(&str, Vec3, f32)> = table
        .iter()
        .map(|(suffix, axis, degrees)| (*suffix, axis_vector(*axis), *degrees))
        .collect();
    deltas.extend(
        finger_deltas
            .iter()
            .map(|(suffix, axis, degrees)| (suffix.as_str(), *axis, *degrees)),
    );
    build_model_space_keyframe(scene, model, rest_rotations, bones, &deltas)
}

const HANG: &[(&str, char, f32)] = &[
    ("LeftArm", 'z', 75.0),
    ("RightArm", 'z', -75.0),
    ("LeftForeArm", 'y', -90.0),
    ("RightForeArm", 'y', 90.0),
    ("LeftForeArm", 'x', -20.0),
    ("RightForeArm", 'x', -20.0),
    ("LeftHand", 'x', 55.0),
    ("RightHand", 'x', 55.0),
    ("LeftUpLeg", 'x', -15.0),
    ("RightUpLeg", 'x', -15.0),
    ("LeftLeg", 'x', 25.0),
    ("RightLeg", 'x', 25.0),
    ("LeftFoot", 'x', -10.0),
    ("RightFoot", 'x', -10.0),
    ("Spine", 'x', 5.0),
];

fn bone_chain(
    bones: &HashMap<String, Entity>,
    root: &str,
    middle: &str,
    end: &str,
) -> Option<IkChain> {
    Some(IkChain {
        root: find_bone(bones, root)?,
        middle: find_bone(bones, middle)?,
        end: find_bone(bones, end)?,
    })
}

pub fn initialize(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    model: Entity,
) {
    let body = sandbox_world.character.body;
    let Some(player) = scene
        .storage
        .get::<nightshade_animation::contract::AnimationPlayer>(model)
    else {
        return;
    };
    let bones = player.bone_name_to_entity.clone();
    let node_index_to_entity = player.node_index_to_entity.clone();
    let clips = player.clips.clone();

    let mut rest = Vec::new();
    for &entity in bones.values() {
        if let Some(transform) = scene
            .storage
            .get::<nightshade_scene::contract::LocalTransform>(entity)
        {
            rest.push((entity, transform.rotation));
        }
    }
    rest.sort_by_key(|(entity, _)| entity.id);
    let rest_rotations: HashMap<Entity, nalgebra_glm::Quat> = rest.iter().copied().collect();

    let mut keyframes = vec![PoseKeyframe {
        name: "rest".to_string(),
        joints: rest.clone(),
    }];

    for clip in &clips {
        let joints = keyframe_from_clip(clip, &bones, &node_index_to_entity);
        if joints.is_empty() {
            continue;
        }
        let name = unique_name(&sanitize_name(&clip.name), &keyframes);
        keyframes.push(PoseKeyframe { name, joints });
    }

    let relaxed_curl = relaxed_finger_curl();
    let grip_curl = grip_finger_curl();

    const IDLE_TABLE: &[(&str, char, f32)] = &[
        ("LeftArm", 'z', -IDLE_ARM_DROP_DEGREES),
        ("RightArm", 'z', IDLE_ARM_DROP_DEGREES),
        ("LeftArm", 'x', -IDLE_ARM_FORWARD_DEGREES),
        ("RightArm", 'x', -IDLE_ARM_FORWARD_DEGREES),
        ("LeftForeArm", 'x', -IDLE_ELBOW_BEND_DEGREES),
        ("RightForeArm", 'x', -IDLE_ELBOW_BEND_DEGREES),
    ];
    keyframes.push(PoseKeyframe {
        name: "idle".to_string(),
        joints: table_keyframe(
            scene,
            model,
            &rest_rotations,
            &bones,
            IDLE_TABLE,
            &relaxed_curl,
        ),
    });

    const REACH_TABLE: &[(&str, char, f32)] = &[
        ("LeftArm", 'y', -REACH_ARM_DEGREES),
        ("RightArm", 'y', REACH_ARM_DEGREES),
    ];
    keyframes.push(PoseKeyframe {
        name: "reach".to_string(),
        joints: table_keyframe(
            scene,
            model,
            &rest_rotations,
            &bones,
            REACH_TABLE,
            &relaxed_curl,
        ),
    });

    let crouch_joints =
        table_keyframe(scene, model, &rest_rotations, &bones, CROUCH, &relaxed_curl);
    let crouch_index = keyframes.len();
    keyframes.push(PoseKeyframe {
        name: "crouch".to_string(),
        joints: crouch_joints,
    });

    let air_rise_index = keyframes.len();
    keyframes.push(PoseKeyframe {
        name: "leap".to_string(),
        joints: table_keyframe(
            scene,
            model,
            &rest_rotations,
            &bones,
            AIR_RISE,
            &relaxed_curl,
        ),
    });
    let air_fall_index = keyframes.len();
    keyframes.push(PoseKeyframe {
        name: "fall".to_string(),
        joints: table_keyframe(
            scene,
            model,
            &rest_rotations,
            &bones,
            AIR_FALL,
            &relaxed_curl,
        ),
    });
    let tuck_forward_index = keyframes.len();
    keyframes.push(PoseKeyframe {
        name: "tuck".to_string(),
        joints: table_keyframe(
            scene,
            model,
            &rest_rotations,
            &bones,
            TUCK_FORWARD,
            &relaxed_curl,
        ),
    });
    let tuck_side_index = keyframes.len();
    keyframes.push(PoseKeyframe {
        name: "tuck_side".to_string(),
        joints: table_keyframe(
            scene,
            model,
            &rest_rotations,
            &bones,
            TUCK_SIDE,
            &relaxed_curl,
        ),
    });
    let hang_index = keyframes.len();
    keyframes.push(PoseKeyframe {
        name: "hang".to_string(),
        joints: table_keyframe(scene, model, &rest_rotations, &bones, HANG, &grip_curl),
    });

    let hips = find_bone(&bones, "Hips");
    let (hips_rest_translation, hips_drop_local) = hips
        .map(|hips_entity| {
            let rest_translation = scene
                .storage
                .get::<nightshade_scene::contract::LocalTransform>(hips_entity)
                .map(|transform| transform.translation)
                .unwrap_or_else(Vec3::zeros);
            (
                rest_translation,
                hips_local_drop_vector(scene, model, hips_entity),
            )
        })
        .unwrap_or((Vec3::zeros(), Vec3::zeros()));

    let pose = &mut sandbox_world.pose;
    pose.keyframes = keyframes;
    pose.rest_rotations = rest_rotations;
    pose.gait_tracks = build_gait_tracks(&bones);
    pose.crouch_index = Some(crouch_index);
    pose.air_rise_index = Some(air_rise_index);
    pose.air_fall_index = Some(air_fall_index);
    pose.tuck_forward_index = Some(tuck_forward_index);
    pose.tuck_side_index = Some(tuck_side_index);
    pose.hang_index = Some(hang_index);
    pose.leg_chains = [
        bone_chain(&bones, "LeftUpLeg", "LeftLeg", "LeftFoot"),
        bone_chain(&bones, "RightUpLeg", "RightLeg", "RightFoot"),
    ]
    .into_iter()
    .flatten()
    .collect();
    pose.arm_chains = [
        bone_chain(&bones, "LeftArm", "LeftForeArm", "LeftHand"),
        bone_chain(&bones, "RightArm", "RightForeArm", "RightHand"),
    ]
    .into_iter()
    .flatten()
    .collect();
    pose.hips = hips;
    pose.hips_rest_translation = hips_rest_translation;
    pose.hips_drop_local = hips_drop_local;
    pose.look_joints = LOOK_JOINTS
        .iter()
        .filter_map(|(suffix, weight)| Some((find_bone(&bones, suffix)?, *weight)))
        .collect();
    pose.idle_joints = IDLE_LIFE
        .iter()
        .filter_map(|(suffix, axis, breath, sway)| {
            Some((
                find_bone(&bones, suffix)?,
                axis_vector(*axis),
                *breath,
                *sway,
            ))
        })
        .collect();
    pose.soft_joints = SOFT_JOINTS
        .iter()
        .filter_map(|(suffix, stiffness, damping_ratio)| {
            Some(SoftJoint {
                entity: find_bone(&bones, suffix)?,
                stiffness: *stiffness,
                damping: damping_ratio * 2.0 * stiffness.sqrt(),
                rotation: nalgebra_glm::quat_identity(),
                angular_velocity: Vec3::zeros(),
                initialized: false,
            })
        })
        .collect();
    pose.feet = pose
        .leg_chains
        .iter()
        .enumerate()
        .map(|(index, chain)| FootPlant {
            chain: *chain,
            phase_offset: index as f32 * std::f32::consts::PI,
            in_stance: false,
            initialized: false,
            plant_ball: Vec3::zeros(),
            plant_normal: Vec3::y(),
            stance_ankle: Vec3::zeros(),
            stance_pitch: 0.0,
            lifted: false,
            lift_phase: 0.0,
            last_phase: 0.0,
            lift_offset: Vec3::zeros(),
            lift_height: 0.0,
            lift_pitch: 0.0,
            flat_rotation: crate::systems::world::ik::rotation_below(scene, model, chain.end)
                .unwrap_or_else(nalgebra_glm::quat_identity),
        })
        .collect();
    if let Some(measurements) =
        body.and_then(|body| measure_legs(scene, body, &bones, pose.leg_chains.first()?))
    {
        pose.leg_length = measurements.leg_length;
        pose.ankle_height = measurements.ankle_height;
        pose.toe_height = measurements.toe_height;
        pose.foot_length = measurements.foot_length;
    }
}

struct LegMeasurements {
    leg_length: f32,
    ankle_height: f32,
    toe_height: f32,
    foot_length: f32,
}

fn measure_legs(
    scene: &crate::commands::Scene<'_, '_>,
    body: Entity,
    bones: &HashMap<String, Entity>,
    chain: &IkChain,
) -> Option<LegMeasurements> {
    let floor = crate::systems::world::ik::world_pose_of(scene, body, body)?
        .position
        .y;
    let hip = crate::systems::world::ik::world_pose_of(scene, body, chain.root)?.position;
    let knee = crate::systems::world::ik::world_pose_of(scene, body, chain.middle)?.position;
    let ankle = crate::systems::world::ik::world_pose_of(scene, body, chain.end)?.position;
    let leg_length = (knee - hip).norm() + (ankle - knee).norm();
    let toe = find_bone(bones, "LeftToeBase")
        .and_then(|toe| crate::systems::world::ik::world_pose_of(scene, body, toe))
        .map(|pose| pose.position);
    let (toe_height, foot_length) = match toe {
        Some(toe) => ((toe.y - floor).max(0.0), (toe - ankle).norm()),
        None => (0.0, leg_length * FALLBACK_FOOT_LENGTH_FRACTION),
    };
    Some(LegMeasurements {
        leg_length,
        ankle_height: (ankle.y - floor).max(toe_height + 0.01),
        toe_height,
        foot_length,
    })
}

pub fn cycle_phase(stride_phase: f32) -> f32 {
    stride_phase + GAIT_PHASE_LEAD
}

pub fn duty_factor(run_factor: f32, crouch_factor: f32) -> f32 {
    let upright = WALK_DUTY + (RUN_DUTY - WALK_DUTY) * run_factor.clamp(0.0, 1.0);
    (upright + (CROUCH_DUTY - upright) * crouch_factor.clamp(0.0, 1.0)).clamp(0.1, 0.9)
}

pub fn stride_radius(
    leg_length: f32,
    walk_speed_factor: f32,
    run_factor: f32,
    crouch_factor: f32,
) -> f32 {
    let run_factor = run_factor.clamp(0.0, 1.0);
    let crouch_factor = crouch_factor.clamp(0.0, 1.0);
    let walk_travel = WALK_STANCE_TRAVEL_SLOW
        + (WALK_STANCE_TRAVEL - WALK_STANCE_TRAVEL_SLOW) * walk_speed_factor.clamp(0.0, 1.0);
    let upright_travel = walk_travel + (RUN_STANCE_TRAVEL - walk_travel) * run_factor;
    let travel = leg_length * upright_travel * (1.0 + (CROUCH_LEG_FRACTION - 1.0) * crouch_factor);
    let duty = duty_factor(run_factor, crouch_factor);
    (travel / (duty * std::f32::consts::TAU)).max(MINIMUM_STRIDE_RADIUS)
}

pub fn contact_fraction(run_factor: f32) -> f32 {
    WALK_CONTACT_FRACTION
        + (RUN_CONTACT_FRACTION - WALK_CONTACT_FRACTION) * run_factor.clamp(0.0, 1.0)
}

pub fn apply_softness(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
) {
    if sandbox_world.ragdoll.mode != crate::resources::RagdollMode::Off {
        for soft in &mut sandbox_world.pose.soft_joints {
            soft.initialized = false;
        }
        return;
    }
    let delta_time = scene.stage.time.delta_time.min(SOFTNESS_MAX_STEP);
    let substeps =
        ((delta_time / SOFTNESS_SUBSTEP).ceil() as usize).clamp(1, SOFTNESS_MAX_SUBSTEPS);
    let substep = delta_time / substeps as f32;
    let rigid_blend = sandbox_world.character.hang_weight.clamp(0.0, 1.0);
    for soft in &mut sandbox_world.pose.soft_joints {
        let Some(target) = scene
            .storage
            .get::<nightshade_scene::contract::LocalTransform>(soft.entity)
            .map(|transform| transform.rotation)
        else {
            continue;
        };
        if !soft.initialized {
            soft.rotation = target;
            soft.angular_velocity = Vec3::zeros();
            soft.initialized = true;
            continue;
        }
        let mut target = target;
        if nalgebra_glm::quat_dot(&soft.rotation, &target) < 0.0 {
            target = -target;
        }
        for _ in 0..substeps {
            let error =
                quat_to_rotation_vector(&(target * nalgebra_glm::quat_inverse(&soft.rotation)));
            soft.angular_velocity +=
                (error * soft.stiffness - soft.angular_velocity * soft.damping) * substep;
            let step = rotation_vector_to_quat(&(soft.angular_velocity * substep));
            soft.rotation = nalgebra_glm::quat_normalize(&(step * soft.rotation));
        }
        if rigid_blend > 1e-3 {
            soft.rotation = nalgebra_glm::quat_normalize(
                &(soft.rotation * (1.0 - rigid_blend) + target * rigid_blend),
            );
        }
        if let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(soft.entity)
        {
            transform.rotation = soft.rotation;
        }
    }
}

pub fn quat_to_rotation_vector(rotation: &nalgebra_glm::Quat) -> Vec3 {
    let mut rotation = *rotation;
    if rotation.w < 0.0 {
        rotation = -rotation;
    }
    let half_angle = rotation.w.clamp(-1.0, 1.0).acos();
    let sine = (1.0 - rotation.w * rotation.w).max(0.0).sqrt();
    if sine < 1e-5 {
        return Vec3::new(rotation.i, rotation.j, rotation.k) * 2.0;
    }
    Vec3::new(rotation.i, rotation.j, rotation.k) * (2.0 * half_angle / sine)
}

fn rotation_vector_to_quat(rotation_vector: &Vec3) -> nalgebra_glm::Quat {
    let angle = rotation_vector.norm();
    if angle < 1e-6 {
        return nalgebra_glm::quat_identity();
    }
    nalgebra_glm::quat_angle_axis(angle, &(rotation_vector / angle))
}

fn hips_local_drop_vector(
    scene: &crate::commands::Scene<'_, '_>,
    model: Entity,
    hips: Entity,
) -> Vec3 {
    let mut stack = vec![(model, nalgebra_glm::Mat3::identity())];
    while let Some((entity, parent_matrix)) = stack.pop() {
        if entity == hips {
            return parent_matrix
                .try_inverse()
                .map(|inverse| inverse * Vec3::new(0.0, -1.0, 0.0))
                .unwrap_or_else(Vec3::zeros);
        }
        let local_matrix = scene
            .storage
            .get::<nightshade_scene::contract::LocalTransform>(entity)
            .map(|transform| {
                nalgebra_glm::quat_to_mat3(&transform.rotation)
                    * nalgebra_glm::diagonal3x3(&transform.scale)
            })
            .unwrap_or_else(nalgebra_glm::Mat3::identity);
        let matrix = parent_matrix * local_matrix;
        for child in query_children(scene.stage.hierarchy, entity) {
            stack.push((child, matrix));
        }
    }
    Vec3::zeros()
}

pub fn set_pose(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    name: &str,
) -> bool {
    let model = sandbox_world.character.model;
    if name == "clip" {
        sandbox_world.pose.active = None;
        if let Some(model) = model
            && let Some(player) = scene
                .storage
                .get_mut::<nightshade_animation::contract::AnimationPlayer>(model)
            && !player.clips.is_empty()
        {
            player.playing = true;
        }
        return true;
    }
    let Some(index) = sandbox_world
        .pose
        .keyframes
        .iter()
        .position(|keyframe| keyframe.name == name)
    else {
        return false;
    };
    sandbox_world.pose.active = Some(index);
    if let Some(model) = model
        && let Some(player) = scene
            .storage
            .get_mut::<nightshade_animation::contract::AnimationPlayer>(model)
    {
        player.playing = false;
    }
    true
}

fn compose_delta(
    deltas: &mut HashMap<Entity, nalgebra_glm::Quat>,
    entity: Entity,
    rotation: nalgebra_glm::Quat,
) {
    deltas
        .entry(entity)
        .and_modify(|existing| *existing = rotation * *existing)
        .or_insert(rotation);
}

pub fn update(sandbox_world: &mut SandboxResources, scene: &mut crate::commands::Scene<'_, '_>) {
    if sandbox_world.ragdoll.mode != crate::resources::RagdollMode::Off {
        return;
    }
    sandbox_world.character.idle_clock = (sandbox_world.character.idle_clock
        + scene.stage.time.delta_time)
        .rem_euclid(IDLE_CLOCK_WRAP);
    let pose = &sandbox_world.pose;
    let Some(active) = pose.active else {
        return;
    };
    let Some(keyframe) = pose.keyframes.get(active) else {
        return;
    };
    let character = &sandbox_world.character;

    let gait_weight = if keyframe.name == "idle" {
        character.gait_weight.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let gait_locals = if gait_weight > 1e-3
        && let Some(model) = character.model
    {
        Some(build_gait_locals(
            scene,
            model,
            &pose.rest_rotations,
            &pose.gait_tracks,
            cycle_phase(character.stride_phase),
            character.gait_run_factor.clamp(0.0, 1.0),
            character.crouch_amount.clamp(0.0, 1.0),
        ))
    } else {
        None
    };

    let crouch_weight = if pose.active == pose.crouch_index {
        0.0
    } else {
        character.crouch_amount.clamp(0.0, CROUCH_MAX_WEIGHT) * (1.0 - gait_weight)
    };
    let crouch_joints = pose
        .crouch_index
        .and_then(|index| pose.keyframes.get(index))
        .filter(|_| crouch_weight > 1e-3)
        .map(|keyframe| keyframe.joints.as_slice());

    let air_weight = character.air_weight.clamp(0.0, 1.0);
    let air_fall = character.air_fall_weight.clamp(0.0, 1.0);
    let air_pair = if air_weight > 1e-3
        && pose.active != pose.air_rise_index
        && pose.active != pose.air_fall_index
    {
        pose.air_rise_index
            .zip(pose.air_fall_index)
            .and_then(|(rise_index, fall_index)| {
                Some((
                    pose.keyframes.get(rise_index)?.joints.as_slice(),
                    pose.keyframes.get(fall_index)?.joints.as_slice(),
                ))
            })
    } else {
        None
    };

    let hang_weight = character.hang_weight.clamp(0.0, 1.0);
    let hang_joints = pose
        .hang_index
        .and_then(|index| pose.keyframes.get(index))
        .filter(|_| hang_weight > 1e-3 && pose.active != pose.hang_index)
        .map(|keyframe| keyframe.joints.as_slice());

    let tuck_weight = character.tuck_weight.clamp(0.0, 1.0);
    let flip_side = character.flip_side.clamp(0.0, 1.0);
    let tuck_pair = if tuck_weight > 1e-3 {
        pose.tuck_forward_index
            .zip(pose.tuck_side_index)
            .and_then(|(forward_index, side_index)| {
                Some((
                    pose.keyframes.get(forward_index)?.joints.as_slice(),
                    pose.keyframes.get(side_index)?.joints.as_slice(),
                ))
            })
    } else {
        None
    };

    let delta_time = scene.stage.time.delta_time;
    let sharpness = POSE_BLEND_SHARPNESS + GAIT_TRACKING_SHARPNESS_BOOST * gait_weight;
    let blend = 1.0 - (-sharpness * delta_time).exp();

    let mut targets: HashMap<Entity, nalgebra_glm::Quat> =
        HashMap::with_capacity(keyframe.joints.len());
    for (entity, base_rotation) in &keyframe.joints {
        let mut target = *base_rotation;
        if let Some(gait_locals) = &gait_locals
            && let Some(gait_rotation) = gait_locals.get(entity)
        {
            target = blend_rotation(target, *gait_rotation, gait_weight);
        }
        if let Some((rise_joints, fall_joints)) = air_pair
            && let Some(rise_rotation) = joint_rotation(rise_joints, *entity)
            && let Some(fall_rotation) = joint_rotation(fall_joints, *entity)
        {
            let air_rotation = blend_rotation(rise_rotation, fall_rotation, air_fall);
            target = blend_rotation(target, air_rotation, air_weight);
        }
        if let Some(crouch_joints) = crouch_joints
            && let Some(crouch_rotation) = joint_rotation(crouch_joints, *entity)
        {
            target = blend_rotation(target, crouch_rotation, crouch_weight);
        }
        if let Some(hang_joints) = hang_joints
            && let Some(hang_rotation) = joint_rotation(hang_joints, *entity)
        {
            target = blend_rotation(target, hang_rotation, hang_weight);
        }
        if let Some((tuck_forward, tuck_side)) = tuck_pair
            && let Some(forward_rotation) = joint_rotation(tuck_forward, *entity)
            && let Some(side_rotation) = joint_rotation(tuck_side, *entity)
        {
            let tuck_rotation = blend_rotation(forward_rotation, side_rotation, flip_side);
            target = blend_rotation(target, tuck_rotation, tuck_weight);
        }
        targets.insert(*entity, target);
    }

    let mut deltas: HashMap<Entity, nalgebra_glm::Quat> = HashMap::new();

    let look_scale = (1.0 - tuck_weight) * (1.0 - hang_weight);
    let look_yaw = character.look_yaw * look_scale;
    let look_pitch = character.look_pitch * look_scale;
    if look_yaw.abs() > 1e-3 || look_pitch.abs() > 1e-3 {
        for (entity, weight) in &pose.look_joints {
            let rotation = nalgebra_glm::quat_angle_axis(look_yaw * weight, &Vec3::y())
                * nalgebra_glm::quat_angle_axis(look_pitch * weight, &Vec3::x());
            compose_delta(&mut deltas, *entity, rotation);
        }
    }

    let idle_weight =
        (1.0 - gait_weight) * (1.0 - air_weight) * (1.0 - hang_weight) * (1.0 - tuck_weight);
    if idle_weight > 1e-3 {
        let clock = character.idle_clock;
        let breath = (std::f32::consts::TAU * BREATH_HERTZ * clock).sin();
        let sway = (std::f32::consts::TAU * SWAY_HERTZ * clock).sin();
        for (entity, axis, breath_degrees, sway_degrees) in &pose.idle_joints {
            let degrees = breath_degrees * breath + sway_degrees * sway;
            let rotation =
                nalgebra_glm::quat_angle_axis((degrees * idle_weight).to_radians(), axis);
            compose_delta(&mut deltas, *entity, rotation);
        }
    }

    if !deltas.is_empty()
        && let Some(model) = character.model
    {
        apply_deltas_to_targets(scene, model, &mut targets, &deltas);
    }

    for (entity, _) in &keyframe.joints {
        let Some(&target) = targets.get(entity) else {
            continue;
        };
        let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(*entity)
        else {
            continue;
        };
        transform.rotation = blend_rotation(transform.rotation, target, blend);
    }

    if let Some(hips) = pose.hips {
        let drop_weight = if pose.active == pose.crouch_index {
            1.0
        } else {
            character.crouch_amount.clamp(0.0, CROUCH_MAX_WEIGHT)
        };
        let target_translation =
            pose.hips_rest_translation + pose.hips_drop_local * (CROUCH_HIPS_DROP * drop_weight);
        if let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(hips)
        {
            transform.translation =
                transform.translation + (target_translation - transform.translation) * blend;
        }
    }
}

fn apply_deltas_to_targets(
    scene: &crate::commands::Scene<'_, '_>,
    model: Entity,
    targets: &mut HashMap<Entity, nalgebra_glm::Quat>,
    deltas: &HashMap<Entity, nalgebra_glm::Quat>,
) {
    let mut stack = vec![(model, nalgebra_glm::quat_identity())];
    while let Some((entity, posed_parent)) = stack.pop() {
        let local = targets
            .get(&entity)
            .copied()
            .or_else(|| {
                scene
                    .storage
                    .get::<nightshade_scene::contract::LocalTransform>(entity)
                    .map(|transform| transform.rotation)
            })
            .unwrap_or_else(nalgebra_glm::quat_identity);
        let new_local = if let Some(delta) = deltas.get(&entity) {
            let adjusted = nalgebra_glm::quat_normalize(
                &(nalgebra_glm::quat_inverse(&posed_parent) * delta * posed_parent * local),
            );
            targets.insert(entity, adjusted);
            adjusted
        } else {
            local
        };
        let posed = posed_parent * new_local;
        for child in query_children(scene.stage.hierarchy, entity) {
            stack.push((child, posed));
        }
    }
}

pub fn joint_rotation(
    joints: &[(Entity, nalgebra_glm::Quat)],
    entity: Entity,
) -> Option<nalgebra_glm::Quat> {
    joints
        .binary_search_by_key(&entity.id, |(joint, _)| joint.id)
        .ok()
        .map(|index| joints[index].1)
}

pub fn blend_rotation(
    from: nalgebra_glm::Quat,
    mut to: nalgebra_glm::Quat,
    weight: f32,
) -> nalgebra_glm::Quat {
    if nalgebra_glm::quat_dot(&from, &to) < 0.0 {
        to = -to;
    }
    nalgebra_glm::quat_normalize(&(from * (1.0 - weight) + to * weight))
}

fn keyframe_from_clip(
    clip: &AnimationClip,
    bones: &HashMap<String, Entity>,
    node_index_to_entity: &HashMap<usize, Entity>,
) -> Vec<(Entity, nalgebra_glm::Quat)> {
    let mut joints = Vec::new();
    for channel in &clip.channels {
        if !matches!(channel.target_property, AnimationProperty::Rotation) {
            continue;
        }
        let entity = channel
            .target_bone_name
            .as_ref()
            .and_then(|name| bones.get(name))
            .or_else(|| node_index_to_entity.get(&channel.target_node));
        let Some(&entity) = entity else {
            continue;
        };
        let rotation = match &channel.sampler.output {
            AnimationSamplerOutput::Quat(values) => values.first().copied(),
            AnimationSamplerOutput::CubicSplineQuat { values, .. } => values.first().copied(),
            _ => None,
        };
        if let Some(rotation) = rotation {
            joints.push((entity, rotation));
        }
    }
    joints
}

fn build_model_space_keyframe(
    scene: &crate::commands::Scene<'_, '_>,
    model: Entity,
    rest_rotations: &HashMap<Entity, nalgebra_glm::Quat>,
    bones: &HashMap<String, Entity>,
    deltas: &[(&str, Vec3, f32)],
) -> Vec<(Entity, nalgebra_glm::Quat)> {
    let mut delta_by_entity: HashMap<Entity, nalgebra_glm::Quat> = HashMap::new();
    for (bone_suffix, axis, degrees) in deltas {
        let Some(entity) = find_bone(bones, bone_suffix) else {
            continue;
        };
        let rotation = nalgebra_glm::quat_angle_axis(degrees.to_radians(), axis);
        delta_by_entity
            .entry(entity)
            .and_modify(|existing| *existing = rotation * *existing)
            .or_insert(rotation);
    }
    let locals = build_locals_from_model_deltas(scene, model, rest_rotations, &delta_by_entity);
    let mut joints: Vec<(Entity, nalgebra_glm::Quat)> = locals.into_iter().collect();
    joints.sort_by_key(|(entity, _)| entity.id);
    joints
}

fn build_locals_from_model_deltas(
    scene: &crate::commands::Scene<'_, '_>,
    model: Entity,
    rest_rotations: &HashMap<Entity, nalgebra_glm::Quat>,
    delta_by_entity: &HashMap<Entity, nalgebra_glm::Quat>,
) -> HashMap<Entity, nalgebra_glm::Quat> {
    let mut locals = HashMap::new();
    let mut stack = vec![(model, nalgebra_glm::quat_identity())];
    while let Some((entity, posed_parent)) = stack.pop() {
        let rest_local = rest_rotations
            .get(&entity)
            .copied()
            .or_else(|| {
                scene
                    .storage
                    .get::<nightshade_scene::contract::LocalTransform>(entity)
                    .map(|transform| transform.rotation)
            })
            .unwrap_or_else(nalgebra_glm::quat_identity);
        let local = if let Some(delta) = delta_by_entity.get(&entity) {
            nalgebra_glm::quat_inverse(&posed_parent) * delta * posed_parent * rest_local
        } else {
            rest_local
        };
        if rest_rotations.contains_key(&entity) {
            locals.insert(entity, local);
        }
        let posed_rotation = posed_parent * local;
        for child in query_children(scene.stage.hierarchy, entity) {
            stack.push((child, posed_rotation));
        }
    }
    locals
}

fn build_gait_tracks(bones: &HashMap<String, Entity>) -> Vec<GaitTrack> {
    let mut track_keys: Vec<(String, char)> = Vec::new();
    for (suffix, axis, _) in GAIT_REACH
        .iter()
        .chain(GAIT_PASS.iter())
        .chain(GAIT_PUSH.iter())
        .chain(WALK_REACH.iter())
        .chain(WALK_PASS.iter())
        .chain(WALK_PUSH.iter())
        .chain(CROUCH_WALK_REACH.iter())
        .chain(CROUCH_WALK_PASS.iter())
        .chain(CROUCH_WALK_PUSH.iter())
    {
        let key = (suffix.to_string(), *axis);
        if !track_keys.contains(&key) {
            track_keys.push(key);
        }
    }

    let mut tracks = Vec::new();
    for (suffix, axis) in track_keys {
        let Some(entity) = find_bone(bones, &suffix) else {
            continue;
        };
        let partner = partner_suffix(&suffix);
        let sign = mirror_sign(axis);
        let run = [
            table_value(GAIT_REACH, &suffix, axis),
            table_value(GAIT_PASS, &suffix, axis),
            table_value(GAIT_PUSH, &suffix, axis),
            table_value(GAIT_REACH, &partner, axis) * sign,
            table_value(GAIT_PASS, &partner, axis) * sign,
            table_value(GAIT_PUSH, &partner, axis) * sign,
        ];
        let walk = [
            table_value(WALK_REACH, &suffix, axis),
            table_value(WALK_PASS, &suffix, axis),
            table_value(WALK_PUSH, &suffix, axis),
            table_value(WALK_REACH, &partner, axis) * sign,
            table_value(WALK_PASS, &partner, axis) * sign,
            table_value(WALK_PUSH, &partner, axis) * sign,
        ];
        let crouch_walk = [
            table_value(CROUCH_WALK_REACH, &suffix, axis),
            table_value(CROUCH_WALK_PASS, &suffix, axis),
            table_value(CROUCH_WALK_PUSH, &suffix, axis),
            table_value(CROUCH_WALK_REACH, &partner, axis) * sign,
            table_value(CROUCH_WALK_PASS, &partner, axis) * sign,
            table_value(CROUCH_WALK_PUSH, &partner, axis) * sign,
        ];
        tracks.push(GaitTrack {
            entity,
            axis: axis_vector(axis),
            run,
            walk,
            crouch_walk,
        });
    }
    tracks
}

fn build_gait_locals(
    scene: &crate::commands::Scene<'_, '_>,
    model: Entity,
    rest_rotations: &HashMap<Entity, nalgebra_glm::Quat>,
    tracks: &[GaitTrack],
    phase: f32,
    run_factor: f32,
    crouch_factor: f32,
) -> HashMap<Entity, nalgebra_glm::Quat> {
    let warped_phase = phase + STRIDE_STANCE_WARP * run_factor * (2.0 * phase).sin();
    let mut delta_by_entity: HashMap<Entity, nalgebra_glm::Quat> = HashMap::new();
    for track in tracks {
        let walk_degrees = sample_cycle(&track.walk, warped_phase);
        let run_degrees = sample_cycle(&track.run, warped_phase);
        let upright_degrees = walk_degrees + (run_degrees - walk_degrees) * run_factor;
        let crouch_degrees = sample_cycle(&track.crouch_walk, warped_phase);
        let degrees = upright_degrees + (crouch_degrees - upright_degrees) * crouch_factor;
        if degrees.abs() < 1e-3 {
            continue;
        }
        let rotation = nalgebra_glm::quat_angle_axis(degrees.to_radians(), &track.axis);
        delta_by_entity
            .entry(track.entity)
            .and_modify(|existing| *existing = rotation * *existing)
            .or_insert(rotation);
    }
    build_locals_from_model_deltas(scene, model, rest_rotations, &delta_by_entity)
}

fn sample_cycle(values: &[f32; 6], phase: f32) -> f32 {
    let normalized = phase.rem_euclid(std::f32::consts::TAU) / (std::f32::consts::TAU / 6.0);
    let segment = (normalized.floor() as usize) % 6;
    let local = normalized - normalized.floor();
    let point_0 = values[(segment + 5) % 6];
    let point_1 = values[segment];
    let point_2 = values[(segment + 1) % 6];
    let point_3 = values[(segment + 2) % 6];
    catmull_rom(point_0, point_1, point_2, point_3, local)
}

fn catmull_rom(point_0: f32, point_1: f32, point_2: f32, point_3: f32, t: f32) -> f32 {
    let t_squared = t * t;
    let t_cubed = t_squared * t;
    0.5 * (2.0 * point_1
        + (point_2 - point_0) * t
        + (2.0 * point_0 - 5.0 * point_1 + 4.0 * point_2 - point_3) * t_squared
        + (3.0 * point_1 - 3.0 * point_2 + point_0 - point_3) * t_cubed)
}

fn table_value(table: &[(&str, char, f32)], suffix: &str, axis: char) -> f32 {
    table
        .iter()
        .find(|(entry_suffix, entry_axis, _)| *entry_suffix == suffix && *entry_axis == axis)
        .map(|(_, _, degrees)| *degrees)
        .unwrap_or(0.0)
}

fn partner_suffix(suffix: &str) -> String {
    if let Some(rest) = suffix.strip_prefix("Left") {
        format!("Right{rest}")
    } else if let Some(rest) = suffix.strip_prefix("Right") {
        format!("Left{rest}")
    } else {
        suffix.to_string()
    }
}

fn axis_vector(axis: char) -> Vec3 {
    match axis {
        'x' => Vec3::x(),
        'y' => Vec3::y(),
        _ => Vec3::z(),
    }
}

fn mirror_sign(axis: char) -> f32 {
    if axis == 'x' { 1.0 } else { -1.0 }
}

pub fn find_bone(bones: &HashMap<String, Entity>, suffix: &str) -> Option<Entity> {
    bones
        .iter()
        .find(|(name, _)| {
            name.as_str() == suffix
                || name
                    .rsplit_once(':')
                    .is_some_and(|(_, short_name)| short_name == suffix)
        })
        .map(|(_, &entity)| entity)
}

fn sanitize_name(clip_name: &str) -> String {
    let sanitized: String = clip_name
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "clip_pose".to_string()
    } else {
        sanitized
    }
}

fn unique_name(base: &str, keyframes: &[PoseKeyframe]) -> String {
    if !keyframes.iter().any(|keyframe| keyframe.name == base) {
        return base.to_string();
    }
    let mut counter = 2;
    loop {
        let candidate = format!("{base}_{counter}");
        if !keyframes.iter().any(|keyframe| keyframe.name == candidate) {
            return candidate;
        }
        counter += 1;
    }
}
