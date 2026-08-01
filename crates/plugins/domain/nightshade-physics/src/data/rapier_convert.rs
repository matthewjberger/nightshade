use crate::contract::*;

pub(crate) fn rigid_body_type_to_rapier(value: RigidBodyType) -> rapier3d::prelude::RigidBodyType {
    match value {
        RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
        RigidBodyType::Fixed => rapier3d::prelude::RigidBodyType::Fixed,
        RigidBodyType::KinematicPositionBased => {
            rapier3d::prelude::RigidBodyType::KinematicPositionBased
        }
        RigidBodyType::KinematicVelocityBased => {
            rapier3d::prelude::RigidBodyType::KinematicVelocityBased
        }
    }
}
pub(crate) fn locked_axes_to_rapier(value: LockedAxes) -> rapier3d::prelude::LockedAxes {
    rapier3d::prelude::LockedAxes::from_bits_truncate(value.bits())
}
pub(crate) fn interaction_groups_to_rapier(
    value: InteractionGroups,
) -> rapier3d::prelude::InteractionGroups {
    rapier3d::prelude::InteractionGroups::new(
        rapier3d::prelude::Group::from_bits_truncate(value.memberships),
        rapier3d::prelude::Group::from_bits_truncate(value.filter),
    )
}
pub(crate) fn rigid_body_handle_from_rapier(
    value: rapier3d::prelude::RigidBodyHandle,
) -> RigidBodyHandle {
    let (index, generation) = value.into_raw_parts();
    RigidBodyHandle(((generation as u64) << 32) | (index as u64))
}
pub(crate) fn rigid_body_handle_to_rapier(
    value: RigidBodyHandle,
) -> rapier3d::prelude::RigidBodyHandle {
    let index = (value.0 & 0xFFFF_FFFF) as u32;
    let generation = ((value.0 >> 32) & 0xFFFF_FFFF) as u32;
    rapier3d::prelude::RigidBodyHandle::from_raw_parts(index, generation)
}
pub(crate) fn collider_handle_to_rapier(
    value: ColliderHandle,
) -> rapier3d::prelude::ColliderHandle {
    let index = (value.0 & 0xFFFF_FFFF) as u32;
    let generation = ((value.0 >> 32) & 0xFFFF_FFFF) as u32;
    rapier3d::prelude::ColliderHandle::from_raw_parts(index, generation)
}
pub(crate) fn character_controller_to_rapier(
    config: &CharacterControllerConfig,
    scale: f32,
) -> rapier3d::control::KinematicCharacterController {
    rapier3d::control::KinematicCharacterController {
        offset: rapier3d::control::CharacterLength::Absolute(config.offset * scale),
        up: rapier3d::na::Vector3::y_axis(),
        max_slope_climb_angle: config.max_slope_climb_angle,
        min_slope_slide_angle: config.min_slope_slide_angle,
        autostep: config.autostep_max_height.map(|max_height| {
            rapier3d::control::CharacterAutostep {
                max_height: rapier3d::control::CharacterLength::Absolute(max_height * scale),
                min_width: rapier3d::control::CharacterLength::Absolute(
                    config.autostep_min_width.unwrap_or(0.2) * scale,
                ),
                include_dynamic_bodies: config.autostep_include_dynamic_bodies,
            }
        }),
        snap_to_ground: config
            .snap_to_ground
            .map(|d| rapier3d::control::CharacterLength::Absolute(d * scale)),
        ..Default::default()
    }
}

pub(crate) fn rigid_body_to_rapier(body: &RigidBodyComponent) -> rapier3d::prelude::RigidBody {
    use rapier3d::prelude::*;

    let translation = vector![
        body.translation[0],
        body.translation[1],
        body.translation[2]
    ];
    let rotation = rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
        body.rotation[3],
        body.rotation[0],
        body.rotation[1],
        body.rotation[2],
    ));
    let position = rapier3d::na::Isometry3::from_parts(translation.into(), rotation);

    let rapier_body_type = rigid_body_type_to_rapier(body.body_type);
    let rapier_locked_axes = locked_axes_to_rapier(body.locked_axes);

    let mut rb = RigidBodyBuilder::new(rapier_body_type)
        .pose(position)
        .linvel(vector![body.linvel[0], body.linvel[1], body.linvel[2]])
        .angvel(vector![body.angvel[0], body.angvel[1], body.angvel[2]])
        .linear_damping(body.linear_damping)
        .angular_damping(body.angular_damping)
        .gravity_scale(body.gravity_scale)
        .locked_axes(rapier_locked_axes)
        .ccd_enabled(body.ccd_enabled)
        .build();

    if body.body_type == crate::contract::types::RigidBodyType::Dynamic {
        rb.set_additional_mass(body.mass, true);
    }

    rb
}
pub(crate) fn collider_to_rapier(collider: &ColliderComponent) -> rapier3d::prelude::Collider {
    use rapier3d::prelude::{ColliderBuilder, DMatrix, Point, Real, SharedShape};

    let shape: SharedShape = match &collider.shape {
        ColliderShape::Ball { radius } => SharedShape::ball(*radius),
        ColliderShape::Cuboid { hx, hy, hz } => SharedShape::cuboid(*hx, *hy, *hz),
        ColliderShape::Capsule {
            half_height,
            radius,
        } => SharedShape::capsule_y(*half_height, *radius),
        ColliderShape::Cylinder {
            half_height,
            radius,
        } => SharedShape::cylinder(*half_height, *radius),
        ColliderShape::Cone {
            half_height,
            radius,
        } => SharedShape::cone(*half_height, *radius),
        ColliderShape::ConvexMesh { vertices } => {
            let points: Vec<Point<Real>> = vertices
                .iter()
                .map(|v| Point::new(v[0], v[1], v[2]))
                .collect();
            SharedShape::convex_hull(&points).unwrap_or_else(|| SharedShape::ball(0.1))
        }
        ColliderShape::TriMesh { vertices, indices } => {
            let points: Vec<Point<Real>> = vertices
                .iter()
                .map(|v| Point::new(v[0], v[1], v[2]))
                .collect();
            SharedShape::trimesh(points, indices.clone()).unwrap_or_else(|_| SharedShape::ball(0.1))
        }
        ColliderShape::HeightField {
            nrows,
            ncols,
            heights,
            scale,
        } => SharedShape::heightfield(
            DMatrix::from_vec(*nrows, *ncols, heights.clone()),
            rapier3d::na::Vector3::new(scale[0], scale[1], scale[2]),
        ),
    };

    let rapier_collision_groups = interaction_groups_to_rapier(collider.collision_groups);
    let rapier_solver_groups = interaction_groups_to_rapier(collider.solver_groups);

    ColliderBuilder::new(shape)
        .friction(collider.friction)
        .restitution(collider.restitution)
        .density(collider.density)
        .sensor(collider.is_sensor)
        .collision_groups(rapier_collision_groups)
        .solver_groups(rapier_solver_groups)
        .build()
}
