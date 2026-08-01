use crate::data::Settings;
use crate::queries::shade;
use nalgebra_glm::{Quat, Vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_scene::prelude::*;
use nightshade_sdf::contract::{
    SdfBlend, SdfCollides, SdfCollisionFocus, SdfMetal, SdfOperation, SdfOrder, SdfRoughness,
    SdfShape, SdfTint,
};

pub fn spawn_edit(
    commands: &mut Commands,
    name: &str,
    local_transform: LocalTransform,
    shape: SdfShape,
) -> Entity {
    commands.spawn((
        Name(name.to_string()),
        local_transform,
        GlobalTransform(local_transform.as_matrix()),
        shape,
    ))
}

pub fn placed(translation: Vec3) -> LocalTransform {
    LocalTransform {
        translation,
        ..Default::default()
    }
}

pub fn dress_edit(
    commands: &mut Commands,
    edit: Entity,
    operation: SdfOperation,
    blend: f32,
    tint: Vec3,
    order: u32,
) {
    commands.set(edit, operation);
    commands.set(edit, SdfBlend(blend));
    commands.set(edit, SdfTint(tint));
    commands.set(edit, SdfOrder(order));
}

pub fn finish_edit(commands: &mut Commands, edit: Entity, roughness: f32, metal: f32) {
    commands.set(edit, SdfRoughness(roughness));
    commands.set(edit, SdfMetal(metal));
}

pub fn spawn_flying_camera(
    commands: &mut Commands,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
) -> Entity {
    let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::new(0.0, 1.0, 0.0))
        * nalgebra_glm::quat_angle_axis(pitch, &Vec3::new(1.0, 0.0, 0.0));
    let local_transform = LocalTransform {
        translation: position,
        rotation,
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    commands.spawn((
        Name("Sdf Camera".to_string()),
        local_transform,
        GlobalTransform(local_transform.as_matrix()),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 55.0_f32.to_radians(),
                z_near: 0.05,
                z_far: None,
            }),
            smoothing: Some(Smoothing {
                move_speed: speed,
                ..Default::default()
            }),
        },
    ))
}

pub fn spawn_ground(
    commands: &mut Commands,
    mesh: nightshade_sdf::contract::render_data::SdfCollisionMesh,
    friction: f32,
) -> Entity {
    commands.spawn((
        Name("Sdf Ground".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        nightshade_physics::prelude::RigidBodyComponent::new_static(),
        nightshade_physics::prelude::ColliderComponent {
            shape: nightshade_physics::prelude::ColliderShape::TriMesh {
                vertices: mesh.vertices,
                indices: mesh.triangles,
            },
            friction,
            ..Default::default()
        },
    ))
}

pub fn spawn_body(
    commands: &mut Commands,
    name: &str,
    local_transform: LocalTransform,
    shape: SdfShape,
    collider: nightshade_physics::prelude::ColliderComponent,
    velocity: Vec3,
    focus: f32,
) -> Entity {
    let position = local_transform.translation;
    let turn = local_transform.rotation;
    let body = spawn_edit(commands, name, local_transform, shape);
    commands.set(
        body,
        nightshade_physics::prelude::RigidBodyComponent {
            translation: [position.x, position.y, position.z],
            rotation: [turn.i, turn.j, turn.k, turn.w],
            linvel: [velocity.x, velocity.y, velocity.z],
            ccd_enabled: true,
            ..nightshade_physics::prelude::RigidBodyComponent::new_dynamic()
        },
    );
    commands.set(body, collider);
    commands.set(
        body,
        nightshade_physics::prelude::PhysicsInterpolation {
            enabled: true,
            ..nightshade_physics::prelude::PhysicsInterpolation::new()
        },
    );
    commands.set(body, SdfCollides(false));
    commands.set(body, SdfCollisionFocus(focus));
    body
}

pub fn spawn_stone(
    commands: &mut Commands,
    config: &Config<Settings>,
    position: Vec3,
    velocity: Vec3,
) -> Entity {
    let radius = config.number(Settings::StoneRadius);
    let stone = spawn_body(
        commands,
        "Stone",
        placed(position),
        SdfShape::Sphere { radius },
        nightshade_physics::prelude::ColliderComponent::new_ball(radius)
            .with_friction(config.number(Settings::StoneFriction))
            .with_restitution(config.number(Settings::StoneRestitution)),
        velocity,
        config.number(Settings::StoneFocus),
    );
    dress_edit(
        commands,
        stone,
        SdfOperation::Union,
        config.number(Settings::StoneBlend),
        shade(config, Settings::StoneTint),
        crate::theme::PROP_ORDER,
    );
    finish_edit(
        commands,
        stone,
        config.number(Settings::StoneRoughness),
        0.0,
    );
    stone
}

pub fn spawn_bar(
    commands: &mut Commands,
    config: &Config<Settings>,
    position: Vec3,
    rotation: Quat,
) -> Entity {
    let length = config.number(Settings::BarLength);
    let width = config.number(Settings::BarWidth);
    let rounding = config.number(Settings::PropRounding);
    let bar = spawn_body(
        commands,
        "Bar",
        LocalTransform {
            translation: position,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SdfShape::Cube {
            half_extents: Vec3::new(length - rounding, width - rounding, width - rounding),
            rounding,
        },
        nightshade_physics::prelude::ColliderComponent::new_cuboid(length, width, width),
        Vec3::zeros(),
        config.number(Settings::PropFocus),
    );
    dress_edit(
        commands,
        bar,
        SdfOperation::Union,
        0.0,
        shade(config, Settings::BarTint),
        crate::theme::PROP_ORDER,
    );
    finish_edit(commands, bar, config.number(Settings::BarRoughness), 1.0);
    bar
}

pub fn spawn_block(
    commands: &mut Commands,
    config: &Config<Settings>,
    position: Vec3,
    rotation: Quat,
) -> Entity {
    let size = config.number(Settings::BlockSize);
    let rounding = config.number(Settings::PropRounding);
    let block = spawn_body(
        commands,
        "Block",
        LocalTransform {
            translation: position,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SdfShape::Cube {
            half_extents: Vec3::repeat(size - rounding),
            rounding,
        },
        nightshade_physics::prelude::ColliderComponent::new_cuboid(size, size, size),
        Vec3::zeros(),
        config.number(Settings::PropFocus),
    );
    dress_edit(
        commands,
        block,
        SdfOperation::Union,
        0.0,
        shade(config, Settings::BlockTint),
        crate::theme::PROP_ORDER,
    );
    finish_edit(
        commands,
        block,
        config.number(Settings::BlockRoughness),
        1.0,
    );
    block
}

pub fn spawn_spike(
    commands: &mut Commands,
    config: &Config<Settings>,
    position: Vec3,
    rotation: Quat,
) -> Entity {
    let radius = config.number(Settings::SpikeRadius);
    let rounding = config.number(Settings::PropRounding);
    let points = [
        [radius, 0.0, 0.0],
        [-radius, 0.0, 0.0],
        [0.0, radius, 0.0],
        [0.0, -radius, 0.0],
        [0.0, 0.0, radius],
        [0.0, 0.0, -radius],
    ];
    let spike = spawn_body(
        commands,
        "Spike",
        LocalTransform {
            translation: position,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SdfShape::Octahedron {
            radius: radius - rounding,
            rounding,
        },
        nightshade_physics::prelude::ColliderComponent {
            shape: nightshade_physics::prelude::ColliderShape::ConvexMesh {
                vertices: points.to_vec(),
            },
            ..Default::default()
        },
        Vec3::zeros(),
        config.number(Settings::PropFocus),
    );
    dress_edit(
        commands,
        spike,
        SdfOperation::Union,
        0.0,
        shade(config, Settings::SpikeTint),
        crate::theme::PROP_ORDER,
    );
    finish_edit(
        commands,
        spike,
        config.number(Settings::SpikeRoughness),
        0.0,
    );
    spike
}

pub fn lay_capsule(
    commands: &mut Commands,
    name: &str,
    from: Vec3,
    to: Vec3,
    radius: f32,
) -> Entity {
    let along = to - from;
    let length = along.magnitude();
    let middle = (from + to) * 0.5;
    let rotation = if length > f32::EPSILON {
        nalgebra_glm::quat_rotation(&Vec3::new(0.0, 1.0, 0.0), &(along / length))
    } else {
        Quat::identity()
    };
    spawn_edit(
        commands,
        name,
        LocalTransform {
            translation: middle,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SdfShape::Capsule {
            half_height: length * 0.5,
            radius,
        },
    )
}

pub fn spawn_anchor(commands: &mut Commands, position: Vec3, focus: f32) -> Entity {
    let local_transform = placed(position);
    commands.spawn((
        Name("Ground Anchor".to_string()),
        local_transform,
        GlobalTransform(local_transform.as_matrix()),
        SdfCollisionFocus(focus),
    ))
}

pub fn carry_along(
    commands: &mut Commands,
    edit: Entity,
    position: Vec3,
    collider: nightshade_physics::prelude::ColliderComponent,
) {
    commands.set(
        edit,
        nightshade_physics::prelude::RigidBodyComponent {
            translation: [position.x, position.y, position.z],
            ..nightshade_physics::prelude::RigidBodyComponent::new_kinematic()
        },
    );
    commands.set(edit, collider);
    commands.set(edit, SdfCollides(false));
}
