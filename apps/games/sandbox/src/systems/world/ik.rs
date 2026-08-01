use crate::resources::{FootPlant, IkChain, SandboxResources};
use crate::systems::world::pose;
use nightshade_ease::smooth_step;
use nightshade_physics::prelude::*;

const FOOT_PROBE_RISE: f32 = 0.5;
const FOOT_PROBE_DISTANCE: f32 = 2.5;
const HAND_EDGE_FORWARD: f32 = 0.5;
const HAND_EDGE_SPREAD: f32 = 0.22;
const HAND_EDGE_LIFT: f32 = 0.03;
const SWING_CLEARANCE_WALK: f32 = 0.04;
const SWING_CLEARANCE_RUN: f32 = 0.18;
const PLANT_SEED_WEIGHT: f32 = 0.05;
const REST_ALIGN_BAND: f32 = 0.15;
const FOOT_ALIGN_TILT_LIMIT: f32 = 0.6;
const PELVIS_DROP_SHARPNESS: f32 = 12.0;
const PELVIS_DROP_LIMIT: f32 = 0.12;
const GAIT_KNEE_SLACK_WALK: f32 = 0.03;
const GAIT_KNEE_SLACK_RUN: f32 = 0.06;
const EARLY_LIFT_MINIMUM_PROGRESS: f32 = 0.25;
const REACH_MARGIN: f32 = 0.99;
const HEEL_LIFT_MAX: f32 = 1.22;
const HEEL_LIFT_STEPS: usize = 12;
const HEEL_STRIKE_PITCH: f32 = 0.2;
const HEEL_STRIKE_SETTLE: f32 = 0.25;
const HEEL_STRIKE_LIFT: f32 = 0.02;
const SEED_LIFT_HEIGHT: f32 = 0.1;
const SEED_LIFT_PITCH: f32 = -0.5;

pub fn update(sandbox_world: &mut SandboxResources, scene: &mut crate::commands::Scene<'_, '_>) {
    if sandbox_world.ragdoll.mode != crate::resources::RagdollMode::Off {
        release_plants(sandbox_world);
        return;
    }
    let character = &sandbox_world.character;
    if !character.third_person {
        release_plants(sandbox_world);
        return;
    }
    let Some(body) = character.body else {
        return;
    };
    if character.model.is_none() {
        return;
    }

    if character.hanging {
        release_plants(sandbox_world);
        solve_hands_to_ledge(sandbox_world, scene, body);
        return;
    }

    if character.tuck_weight > 0.3 {
        release_plants(sandbox_world);
        return;
    }
    solve_legs(sandbox_world, scene, body);
}

pub fn release_plants(sandbox_world: &mut SandboxResources) {
    for foot in &mut sandbox_world.pose.feet {
        foot.initialized = false;
        foot.in_stance = false;
        foot.lifted = false;
    }
    sandbox_world.pose.pelvis_drop = 0.0;
}

struct GaitFrame {
    travel: Vec3,
    right: Vec3,
    gait_weight: f32,
    duty: f32,
    cycle_distance: f32,
    stance_travel: f32,
    contact_ahead: f32,
    clearance: f32,
    reach_limit: f32,
    flat_angle: f32,
    foot_length: f32,
    toe_height: f32,
    ankle_height: f32,
    cycle: f32,
    model_rotation: nalgebra_glm::Quat,
}

#[derive(Clone, Copy)]
enum FootPhase {
    Stance { progress: f32 },
    Swing { progress: f32 },
}

struct FootSolve {
    chain: IkChain,
    target: Vec3,
    rotation: nalgebra_glm::Quat,
    stance: Option<(Vec3, f32)>,
}

fn solve_legs(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    body: Entity,
) {
    let delta_time = scene.stage.time.delta_time;
    let character = &sandbox_world.character;
    let Some(model) = character.model else {
        return;
    };
    let gait_weight = if character.grounded {
        character.gait_weight.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let run_factor = character.gait_run_factor.clamp(0.0, 1.0);
    let crouch_factor = character.crouch_amount.clamp(0.0, 1.0);
    let horizontal = Vec3::new(character.velocity.x, 0.0, character.velocity.z);
    let travel = if horizontal.norm() > 1e-3 {
        horizontal.normalize()
    } else {
        Vec3::new(character.facing_yaw.sin(), 0.0, character.facing_yaw.cos())
    };
    let duty = pose::duty_factor(run_factor, crouch_factor);
    let cycle_distance = std::f32::consts::TAU * character.stride_radius;
    let stance_travel = duty * cycle_distance;
    let leg = &sandbox_world.pose;
    let foot_length = leg.foot_length.max(1e-3);
    let frame = GaitFrame {
        travel,
        right: normalized_or_up(nalgebra_glm::cross(&travel, &Vec3::y())),
        gait_weight,
        duty,
        cycle_distance,
        stance_travel,
        contact_ahead: pose::contact_fraction(run_factor) * stance_travel,
        clearance: SWING_CLEARANCE_WALK + (SWING_CLEARANCE_RUN - SWING_CLEARANCE_WALK) * run_factor,
        reach_limit: leg.leg_length * REACH_MARGIN,
        flat_angle: ((leg.ankle_height - leg.toe_height) / foot_length)
            .clamp(0.0, 1.0)
            .asin(),
        foot_length,
        toe_height: leg.toe_height,
        ankle_height: leg.ankle_height,
        cycle: pose::cycle_phase(character.stride_phase),
        model_rotation: world_pose_of(scene, body, model)
            .map(|pose| pose.rotation)
            .unwrap_or_else(nalgebra_glm::quat_identity),
    };

    update_plants(sandbox_world, scene, body, &frame);

    let (_, deficit) = foot_solves(&sandbox_world.pose.feet, scene, body, &frame);
    let knee_slack = (GAIT_KNEE_SLACK_WALK
        + (GAIT_KNEE_SLACK_RUN - GAIT_KNEE_SLACK_WALK) * run_factor)
        * gait_weight;
    let drop_target = deficit.max(knee_slack).clamp(0.0, PELVIS_DROP_LIMIT);
    let blend = 1.0 - (-PELVIS_DROP_SHARPNESS * delta_time).exp();
    sandbox_world.pose.pelvis_drop += (drop_target - sandbox_world.pose.pelvis_drop) * blend;
    let pelvis_drop = sandbox_world.pose.pelvis_drop;
    if pelvis_drop > 1e-4
        && let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(model)
    {
        transform.translation.y -= pelvis_drop;
    }

    let (solves, _) = foot_solves(&sandbox_world.pose.feet, scene, body, &frame);
    for (foot, solve) in sandbox_world.pose.feet.iter_mut().zip(&solves) {
        if let Some((ankle, pitch)) = solve.stance {
            foot.stance_ankle = ankle;
            foot.stance_pitch = pitch;
        }
    }
    for solve in solves {
        solve_two_bone(
            scene,
            body,
            solve.chain,
            solve.target,
            Some(frame.travel),
            Some(solve.rotation),
        );
    }
}

fn foot_cycle_phase(frame: &GaitFrame, foot: &FootPlant) -> f32 {
    (frame.cycle + foot.phase_offset).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU
}

fn foot_phase(frame: &GaitFrame, foot: &FootPlant) -> FootPhase {
    let phase = foot_cycle_phase(frame, foot);
    if phase < frame.duty && !foot.lifted {
        FootPhase::Stance {
            progress: phase / frame.duty,
        }
    } else {
        let lift_phase = if foot.lifted {
            foot.lift_phase
        } else {
            frame.duty
        };
        FootPhase::Swing {
            progress: ((phase - lift_phase) / (1.0 - lift_phase).max(1e-3)).clamp(0.0, 1.0),
        }
    }
}

fn record_lift(foot: &mut FootPlant, hip: Vec3, lift_phase: f32) {
    let offset = foot.stance_ankle - hip;
    foot.lift_offset = Vec3::new(offset.x, 0.0, offset.z);
    foot.lift_height = foot.stance_ankle.y;
    foot.lift_pitch = foot.stance_pitch;
    foot.lift_phase = lift_phase;
}

fn update_plants(
    sandbox_world: &mut SandboxResources,
    scene: &crate::commands::Scene<'_, '_>,
    body: Entity,
    frame: &GaitFrame,
) {
    let flat_back = frame.foot_length * frame.flat_angle.cos();
    for foot in &mut sandbox_world.pose.feet {
        let Some(hip) = world_pose_of(scene, body, foot.chain.root).map(|pose| pose.position)
        else {
            continue;
        };
        let phase = foot_cycle_phase(frame, foot);
        if phase + 0.5 < foot.last_phase {
            foot.lifted = false;
        }
        foot.last_phase = phase;
        let stale = !foot.initialized || frame.gait_weight < PLANT_SEED_WEIGHT;
        let mut stance_now = phase < frame.duty && !foot.lifted;
        if stance_now {
            let progress = phase / frame.duty;
            if stale || !foot.in_stance {
                let ball = hip
                    + frame.travel
                        * (frame.contact_ahead - progress * frame.stance_travel + flat_back);
                match ground_point(scene, ball) {
                    Some((ground, normal)) => {
                        foot.plant_ball = ground;
                        foot.plant_normal = normal;
                        foot.initialized = true;
                    }
                    None => foot.initialized = false,
                }
            } else if progress > EARLY_LIFT_MINIMUM_PROGRESS
                && (foot.plant_ball - hip).dot(&frame.travel) <= 0.0
                && (ankle_on_ball(frame, foot.plant_ball, HEEL_LIFT_MAX) - hip).norm()
                    > frame.reach_limit
            {
                foot.lifted = true;
                record_lift(foot, hip, phase);
                stance_now = false;
            }
        } else if foot.in_stance && foot.initialized {
            record_lift(foot, hip, frame.duty);
        } else if stale {
            let behind = hip - frame.travel * (frame.stance_travel - frame.contact_ahead);
            match ground_point(scene, behind) {
                Some((ground, _)) => {
                    let offset = behind - hip;
                    foot.lift_offset = Vec3::new(offset.x, 0.0, offset.z);
                    foot.lift_height = ground.y + frame.ankle_height + SEED_LIFT_HEIGHT;
                    foot.lift_pitch = SEED_LIFT_PITCH;
                    foot.lift_phase = frame.duty;
                    foot.lifted = false;
                    foot.initialized = true;
                }
                None => foot.initialized = false,
            }
        }
        foot.in_stance = stance_now;
    }
}

fn foot_solves(
    feet: &[FootPlant],
    scene: &crate::commands::Scene<'_, '_>,
    body: Entity,
    frame: &GaitFrame,
) -> (Vec<FootSolve>, f32) {
    let mut solves = Vec::with_capacity(feet.len());
    let mut deficit = 0.0_f32;
    for foot in feet {
        let Some(hip) = world_pose_of(scene, body, foot.chain.root).map(|pose| pose.position)
        else {
            continue;
        };
        let Some(end) = world_pose_of(scene, body, foot.chain.end) else {
            continue;
        };
        let rest = rest_target(scene, end.position, frame.ankle_height);
        let rest_rotation = sole_alignment(rest.1, rest.2) * end.rotation;
        let stride = if foot.initialized {
            match foot_phase(frame, foot) {
                FootPhase::Stance { progress } => {
                    let (ankle, pitch) = stance_ankle(frame, foot, hip, progress);
                    let rotation = nalgebra_glm::quat_angle_axis(pitch, &frame.right)
                        * sole_alignment(foot.plant_normal, 1.0)
                        * frame.model_rotation
                        * foot.flat_rotation;
                    Some((ankle, rotation, Some((ankle, pitch))))
                }
                FootPhase::Swing { progress } => swing_ankle(scene, frame, foot, hip, progress)
                    .map(|(ankle, pitch)| {
                        let rotation = nalgebra_glm::quat_angle_axis(pitch, &frame.right)
                            * frame.model_rotation
                            * foot.flat_rotation;
                        (ankle, rotation, None)
                    }),
            }
        } else {
            None
        };
        let swinging = stride
            .as_ref()
            .is_some_and(|(_, _, stance)| stance.is_none());
        let (target, rotation, stance) = match stride {
            Some((position, stride_rotation, stance)) => (
                rest.0 + (position - rest.0) * frame.gait_weight,
                pose::blend_rotation(rest_rotation, stride_rotation, frame.gait_weight),
                stance,
            ),
            None => (rest.0, rest_rotation, None),
        };
        if !swinging {
            deficit = deficit.max((target - hip).norm() - frame.reach_limit);
        }
        solves.push(FootSolve {
            chain: foot.chain,
            target,
            rotation,
            stance,
        });
    }
    (solves, deficit)
}

fn ankle_on_ball(frame: &GaitFrame, ball: Vec3, angle: f32) -> Vec3 {
    ball - frame.travel * frame.foot_length * angle.cos()
        + Vec3::y() * (frame.toe_height + frame.foot_length * angle.sin())
}

fn heel_lift(frame: &GaitFrame, ball: Vec3, hip: Vec3) -> f32 {
    let distance = |angle: f32| (ankle_on_ball(frame, ball, angle) - hip).norm();
    if distance(frame.flat_angle) <= frame.reach_limit {
        return frame.flat_angle;
    }
    if distance(HEEL_LIFT_MAX) > frame.reach_limit {
        return HEEL_LIFT_MAX;
    }
    let mut low = frame.flat_angle;
    let mut high = HEEL_LIFT_MAX;
    for _ in 0..HEEL_LIFT_STEPS {
        let middle = 0.5 * (low + high);
        if distance(middle) > frame.reach_limit {
            low = middle;
        } else {
            high = middle;
        }
    }
    high
}

fn stance_ankle(frame: &GaitFrame, foot: &FootPlant, hip: Vec3, progress: f32) -> (Vec3, f32) {
    let ball_ahead = (foot.plant_ball - hip).dot(&frame.travel) > 0.0;
    let angle = if ball_ahead {
        frame.flat_angle
    } else {
        heel_lift(frame, foot.plant_ball, hip)
    };
    let ankle = ankle_on_ball(frame, foot.plant_ball, angle);
    let strike = HEEL_STRIKE_PITCH * (1.0 - smooth_step((progress / HEEL_STRIKE_SETTLE).min(1.0)));
    (ankle, strike - (angle - frame.flat_angle))
}

fn swing_ankle(
    scene: &crate::commands::Scene<'_, '_>,
    frame: &GaitFrame,
    foot: &FootPlant,
    hip: Vec3,
    progress: f32,
) -> Option<(Vec3, f32)> {
    let remaining = 1.0 - foot_cycle_phase(frame, foot);
    let probe = hip + frame.travel * (frame.contact_ahead + remaining * frame.cycle_distance);
    let (ground, _) = ground_point(scene, probe)?;
    let landing_height = ground.y + frame.ankle_height + HEEL_STRIKE_LIFT;
    let ease = smooth_step(progress);
    let landing_offset = frame.travel * frame.contact_ahead;
    let offset = foot.lift_offset + (landing_offset - foot.lift_offset) * ease;
    let mut position = Vec3::new(
        hip.x + offset.x,
        foot.lift_height + (landing_height - foot.lift_height) * ease,
        hip.z + offset.z,
    );
    position.y += frame.clearance * (std::f32::consts::PI * progress).sin();
    if let Some((floor, _)) = ground_point(scene, position) {
        position.y = position.y.max(floor.y + frame.ankle_height);
    }
    let pitch = foot.lift_pitch + (HEEL_STRIKE_PITCH - foot.lift_pitch) * ease;
    Some((position, pitch))
}

fn rest_target(
    scene: &crate::commands::Scene<'_, '_>,
    ankle: Vec3,
    ankle_height: f32,
) -> (Vec3, Vec3, f32) {
    let Some((ground, normal)) = ground_point(scene, ankle) else {
        return (ankle, Vec3::y(), 0.0);
    };
    let grounded_ankle = ground + Vec3::y() * ankle_height;
    if ankle.y >= grounded_ankle.y + REST_ALIGN_BAND {
        return (ankle, Vec3::y(), 0.0);
    }
    if ankle.y >= grounded_ankle.y {
        return (ankle, normal, 1.0);
    }
    (grounded_ankle, normal, 1.0)
}

fn ground_point(scene: &crate::commands::Scene<'_, '_>, position: Vec3) -> Option<(Vec3, Vec3)> {
    let origin = position + Vec3::new(0.0, FOOT_PROBE_RISE, 0.0);
    let hit = physics_world_cast_ray(
        scene.stage.physics,
        origin,
        Vec3::new(0.0, -1.0, 0.0),
        FOOT_PROBE_DISTANCE,
        None,
    )?;
    Some((Vec3::new(position.x, hit.point.y, position.z), hit.normal))
}

fn normalized_or_up(normal: Vec3) -> Vec3 {
    if normal.norm() > 1e-4 {
        normal.normalize()
    } else {
        Vec3::y()
    }
}

fn sole_alignment(normal: Vec3, weight: f32) -> nalgebra_glm::Quat {
    let full = rotation_between(Vec3::y(), normalized_or_up(normal));
    let rotation_vector = pose::quat_to_rotation_vector(&full);
    let angle = rotation_vector.norm();
    if angle < 1e-5 {
        return nalgebra_glm::quat_identity();
    }
    let limited = (angle * weight.clamp(0.0, 1.0)).min(FOOT_ALIGN_TILT_LIMIT);
    nalgebra_glm::quat_angle_axis(limited, &(rotation_vector / angle))
}

fn solve_hands_to_ledge(
    sandbox_world: &SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    body: Entity,
) {
    let character = &sandbox_world.character;
    let Some(feet) = scene
        .storage
        .get::<nightshade_scene::contract::LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let wall_direction = -character.hang_normal;
    let hand_height = character.hang_edge_y + HAND_EDGE_LIFT;
    let hand_center = feet + wall_direction * HAND_EDGE_FORWARD;
    let arm_chains = sandbox_world.pose.arm_chains.clone();
    for (chain_index, chain) in arm_chains.into_iter().enumerate() {
        let side = if chain_index == 0 { -1.0 } else { 1.0 };
        let target = Vec3::new(
            hand_center.x + character.hang_tangent.x * HAND_EDGE_SPREAD * side,
            hand_height,
            hand_center.z + character.hang_tangent.z * HAND_EDGE_SPREAD * side,
        );
        solve_two_bone(scene, body, chain, target, None, None);
    }
}

pub struct WorldPose {
    pub rotation: nalgebra_glm::Quat,
    pub position: Vec3,
    pub scale: Vec3,
}

fn compose(
    parent: &WorldPose,
    entity: Entity,
    scene: &crate::commands::Scene<'_, '_>,
) -> Option<WorldPose> {
    let local = scene
        .storage
        .get::<nightshade_scene::contract::LocalTransform>(entity)?;
    Some(WorldPose {
        position: parent.position
            + nalgebra_glm::quat_rotate_vec3(
                &parent.rotation,
                &parent.scale.component_mul(&local.translation),
            ),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale.component_mul(&local.scale),
    })
}

fn ancestry(scene: &crate::commands::Scene<'_, '_>, body: Entity, entity: Entity) -> Vec<Entity> {
    let mut chain = vec![entity];
    let mut current = entity;
    while let Some(parent) = scene
        .storage
        .get::<nightshade::ecs::ChildOf>(current)
        .map(|child_of| child_of.0)
    {
        chain.push(parent);
        if parent == body {
            break;
        }
        current = parent;
    }
    chain.reverse();
    chain
}

pub fn world_pose_of(
    scene: &crate::commands::Scene<'_, '_>,
    body: Entity,
    entity: Entity,
) -> Option<WorldPose> {
    let path = ancestry(scene, body, entity);
    if path.first() != Some(&body) {
        return None;
    }
    let mut pose = WorldPose {
        rotation: nalgebra_glm::quat_identity(),
        position: Vec3::zeros(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    for &step in &path {
        pose = compose(&pose, step, scene)?;
    }
    Some(pose)
}

pub fn rotation_below(
    scene: &crate::commands::Scene<'_, '_>,
    root: Entity,
    entity: Entity,
) -> Option<nalgebra_glm::Quat> {
    let path = ancestry(scene, root, entity);
    if path.first() != Some(&root) {
        return None;
    }
    let mut rotation = nalgebra_glm::quat_identity();
    for &step in &path[1..] {
        let local = scene
            .storage
            .get::<nightshade_scene::contract::LocalTransform>(step)?;
        rotation *= local.rotation;
    }
    Some(rotation)
}

fn solve_two_bone(
    scene: &mut crate::commands::Scene<'_, '_>,
    body: Entity,
    chain: IkChain,
    target: Vec3,
    pole: Option<Vec3>,
    end_rotation: Option<nalgebra_glm::Quat>,
) {
    let path = ancestry(scene, body, chain.end);
    if path.first() != Some(&body) {
        return;
    }

    let mut pose = WorldPose {
        rotation: nalgebra_glm::quat_identity(),
        position: Vec3::zeros(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let mut root_parent_rotation = nalgebra_glm::quat_identity();
    let mut root_pose = None;
    let mut middle_pose = None;
    let mut end_pose = None;
    for &step in &path {
        if step == chain.root {
            root_parent_rotation = pose.rotation;
        }
        let Some(next) = compose(&pose, step, scene) else {
            return;
        };
        pose = next;
        if step == chain.root {
            root_pose = Some(WorldPose {
                rotation: pose.rotation,
                position: pose.position,
                scale: pose.scale,
            });
        } else if step == chain.middle {
            middle_pose = Some(WorldPose {
                rotation: pose.rotation,
                position: pose.position,
                scale: pose.scale,
            });
        } else if step == chain.end {
            end_pose = Some(WorldPose {
                rotation: pose.rotation,
                position: pose.position,
                scale: pose.scale,
            });
        }
    }
    let (Some(root_pose), Some(middle_pose), Some(end_pose)) = (root_pose, middle_pose, end_pose)
    else {
        return;
    };

    let hip = root_pose.position;
    let knee = middle_pose.position;
    let ankle = end_pose.position;
    let upper_length = (knee - hip).norm();
    let lower_length = (ankle - knee).norm();
    if upper_length < 1e-4 || lower_length < 1e-4 {
        return;
    }

    let reach = (target - hip).norm().clamp(
        (upper_length - lower_length).abs() + 0.01,
        (upper_length + lower_length) * 0.999,
    );
    let target_direction = (target - hip).normalize();
    let clamped_target = hip + target_direction * reach;

    let (new_root_world, new_middle_world) = match pole {
        Some(pole) => {
            let knee_target = knee_toward_pole(
                hip,
                knee,
                target_direction,
                reach,
                upper_length,
                lower_length,
                pole,
            );
            let root_delta = rotation_between(knee - hip, knee_target - hip);
            let shin = nalgebra_glm::quat_rotate_vec3(&root_delta, &(ankle - knee));
            let middle_delta = rotation_between(shin, clamped_target - knee_target);
            (
                root_delta * root_pose.rotation,
                middle_delta * root_delta * middle_pose.rotation,
            )
        }
        None => {
            let desired_cosine = ((upper_length * upper_length + lower_length * lower_length
                - reach * reach)
                / (2.0 * upper_length * lower_length))
                .clamp(-1.0, 1.0);
            let desired_angle = desired_cosine.acos();
            let to_hip = (hip - knee).normalize();
            let to_ankle = (ankle - knee).normalize();
            let current_angle = to_hip.dot(&to_ankle).clamp(-1.0, 1.0).acos();
            let mut hinge = nalgebra_glm::cross(&to_hip, &to_ankle);
            if hinge.norm() < 1e-4 {
                hinge = nalgebra_glm::quat_rotate_vec3(&middle_pose.rotation, &Vec3::x());
            }
            let hinge = hinge.normalize();
            let middle_delta = nalgebra_glm::quat_angle_axis(desired_angle - current_angle, &hinge);
            let bent_ankle = knee + nalgebra_glm::quat_rotate_vec3(&middle_delta, &(ankle - knee));
            let root_delta = rotation_between(bent_ankle - hip, clamped_target - hip);
            (
                root_delta * root_pose.rotation,
                root_delta * middle_delta * middle_pose.rotation,
            )
        }
    };

    let new_root_local = nalgebra_glm::quat_inverse(&root_parent_rotation) * new_root_world;
    let new_middle_local = nalgebra_glm::quat_inverse(&new_root_world) * new_middle_world;
    let end_world = end_rotation.unwrap_or(end_pose.rotation);
    let new_end_local = nalgebra_glm::quat_inverse(&new_middle_world) * end_world;

    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(chain.root)
    {
        transform.rotation = nalgebra_glm::quat_normalize(&new_root_local);
    }
    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(chain.middle)
    {
        transform.rotation = nalgebra_glm::quat_normalize(&new_middle_local);
    }
    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(chain.end)
    {
        transform.rotation = nalgebra_glm::quat_normalize(&new_end_local);
    }
}

fn knee_toward_pole(
    hip: Vec3,
    knee: Vec3,
    target_direction: Vec3,
    reach: f32,
    upper_length: f32,
    lower_length: f32,
    pole: Vec3,
) -> Vec3 {
    let mut bend = pole - target_direction * pole.dot(&target_direction);
    if bend.norm() < 1e-3 {
        let current = knee - hip;
        bend = current - target_direction * current.dot(&target_direction);
    }
    if bend.norm() < 1e-3 {
        bend = nalgebra_glm::cross(&target_direction, &Vec3::x());
    }
    if bend.norm() < 1e-3 {
        bend = nalgebra_glm::cross(&target_direction, &Vec3::z());
    }
    let bend = bend.normalize();
    let along =
        (upper_length * upper_length - lower_length * lower_length + reach * reach) / (2.0 * reach);
    let across = (upper_length * upper_length - along * along)
        .max(0.0)
        .sqrt();
    hip + target_direction * along + bend * across
}

pub fn rotation_between(from: Vec3, to: Vec3) -> nalgebra_glm::Quat {
    let from_length = from.norm();
    let to_length = to.norm();
    if from_length < 1e-5 || to_length < 1e-5 {
        return nalgebra_glm::quat_identity();
    }
    let from_unit = from / from_length;
    let to_unit = to / to_length;
    let cosine = from_unit.dot(&to_unit).clamp(-1.0, 1.0);
    let axis = nalgebra_glm::cross(&from_unit, &to_unit);
    if axis.norm() < 1e-6 {
        if cosine > 0.0 {
            return nalgebra_glm::quat_identity();
        }
        let fallback = if from_unit.x.abs() < 0.9 {
            Vec3::x()
        } else {
            Vec3::y()
        };
        let perpendicular = nalgebra_glm::cross(&from_unit, &fallback).normalize();
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &perpendicular);
    }
    nalgebra_glm::quat_angle_axis(cosine.acos(), &axis.normalize())
}
