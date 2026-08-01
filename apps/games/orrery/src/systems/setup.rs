use crate::components::{
    Anchor, Autopilot, Body, Found, Grounded, Inside, Landed, Offset, Spin, StandingOn, Target,
    Velocity,
};
use crate::data;
use crate::queries;
use crate::resources::{Arguments, OrreryResources};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};

const GOLDEN_ANGLE: f32 = 2.399_963_2;

pub fn initialize(
    arguments: Res<Arguments>,
    mut resources: ResMut<OrreryResources>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Space;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.12;

    let home = starting_body(&arguments);
    spawn_star_light(&mut commands);
    spawn_bodies(&mut registry, &mut commands);
    spawn_beacons(&mut registry, &mut commands);

    let ship = spawn_ship(home, &mut registry, &mut commands);
    let walker = spawn_walker(home, &mut registry, &mut commands);
    let camera = spawn_chase_camera(home, &mut commands);

    resources.scene.ship = ship;
    resources.scene.plume = spawn_plume(&mut commands);
    resources.scene.walker = walker;
    resources.scene.camera = camera;
    crate::commands::set_signal_total(&mut resources, data::BEACONS.len() as u32);
}

fn starting_body(arguments: &Arguments) -> usize {
    let Some(wanted) = arguments.at.as_deref() else {
        return data::PLANET;
    };
    data::BODIES
        .iter()
        .position(|definition| {
            definition.radius > 0.0 && definition.name.eq_ignore_ascii_case(wanted)
        })
        .unwrap_or(data::PLANET)
}

fn ship_start(home: usize) -> (Vec3, Vec3, Quat) {
    let direction = queries::normalized_or(
        Vec3::new(
            data::SHIP_START_OFFSET[0],
            data::SHIP_START_OFFSET[1],
            data::SHIP_START_OFFSET[2],
        ),
        Vec3::y(),
    );
    let range = queries::body_radius(home) * 3.0 + 12.0;
    let offset = direction * range;
    let tangent = queries::normalized_or(direction.cross(&Vec3::y()), Vec3::x());
    let speed = (data::BODIES[home].gravity / range).sqrt();
    let velocity = queries::body_velocity(home, 0.0) + tangent * speed;
    let facing = queries::normalized_or(tangent - direction * 3.0, tangent);
    let rotation = queries::basis_rotation(facing, direction);
    (offset, velocity, rotation)
}

fn spawn_star_light(commands: &mut Commands) {
    commands.spawn((
        Name("Star Light".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: Vec3::new(
                data::STAR_LIGHT_COLOR[0],
                data::STAR_LIGHT_COLOR[1],
                data::STAR_LIGHT_COLOR[2],
            ),
            intensity: data::STAR_LIGHT_INTENSITY,
            range: data::STAR_LIGHT_RANGE,
            ..Default::default()
        },
    ));
}

fn spawn_bodies(registry: &mut MaterialRegistry, commands: &mut Commands) {
    for (index, definition) in data::BODIES.iter().enumerate() {
        let position = queries::body_position(index, 0.0);
        let transform = LocalTransform::from_translation(position);
        let root = commands.spawn((
            Name(definition.name.to_string()),
            transform,
            GlobalTransform(transform.as_matrix()),
            Body(index),
        ));

        let material = insert_material(
            registry,
            format!("orrery_body_{index}"),
            Material {
                base_color: definition.color,
                emissive_factor: definition.emissive,
                roughness: definition.roughness,
                metallic: 0.0,
                unlit: definition.emissive[0] > 1.0,
                alpha_mode: if definition.color[3] < 1.0 {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                double_sided: definition.color[3] < 1.0,
                ..Default::default()
            },
        );

        if definition.drawn {
            let mesh = spawn_mesh_at(
                commands,
                "Sphere",
                Vec3::zeros(),
                Vec3::repeat(definition.radius),
            );
            commands.set(mesh, MaterialRef::new(material));
            commands.set(mesh, ChildOf(root));
        }

        if definition.inner_radius > 0.0 {
            spawn_shell(commands, root, material, definition);
        }

        if definition.drag > 0.0 {
            spawn_rubble(registry, commands, root, index);
        }

        if index == data::CINDER {
            spawn_tower(registry, commands, root, definition.radius);
        }
    }
}

fn spawn_shell(
    commands: &mut Commands,
    root: Entity,
    material: nightshade_render::data::asset_id::MaterialId,
    definition: &data::BodyDefinition,
) {
    let shell = (definition.radius + definition.inner_radius) * 0.5;
    for step in 0..data::SHELL_POINTS {
        let height = 1.0 - 2.0 * (step as f32 + 0.5) / data::SHELL_POINTS as f32;
        let ring = (1.0 - height * height).max(0.0).sqrt();
        let angle = GOLDEN_ANGLE * step as f32;
        let point = Vec3::new(angle.cos() * ring, height, angle.sin() * ring);
        if point.y > definition.aperture {
            continue;
        }
        let stone = spawn_mesh_at(
            commands,
            "Sphere",
            point * shell,
            Vec3::repeat(data::SHELL_STONE),
        );
        commands.set(stone, MaterialRef::new(material));
        commands.set(stone, ChildOf(root));
    }
}

fn spawn_rubble(
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
    root: Entity,
    index: usize,
) {
    let material = insert_material(
        registry,
        format!("orrery_rubble_{index}"),
        Material {
            base_color: data::RUBBLE_COLOR,
            roughness: 0.95,
            ..Default::default()
        },
    );
    let mut rng = fastrand::Rng::with_seed(0x9E37_79B9);
    let mut signed = move || rng.f32() * 2.0 - 1.0;
    for step in 0..data::RUBBLE_COUNT {
        let angle = GOLDEN_ANGLE * step as f32;
        let span = data::RUBBLE_OUTER - data::RUBBLE_INNER;
        let radius = data::RUBBLE_INNER + signed().abs() * span;
        let lift = signed() * 2.4;
        let point = Vec3::new(angle.cos() * radius, lift, angle.sin() * radius);
        let size = data::RUBBLE_SIZE * (1.0 + signed() * 0.5);
        let stone = spawn_mesh_at(commands, "Cube", point, Vec3::repeat(size));
        commands.set(stone, MaterialRef::new(material));
        commands.set(stone, ChildOf(root));
        commands.set(stone, Spin(signed() * 0.8));
    }
}

fn spawn_tower(
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
    root: Entity,
    radius: f32,
) {
    let material = insert_material(
        registry,
        "orrery_tower",
        Material {
            base_color: data::TOWER_COLOR,
            roughness: 0.8,
            ..Default::default()
        },
    );
    let tower = spawn_mesh_at(
        commands,
        "Cylinder",
        Vec3::y() * (radius + data::TOWER_HEIGHT * 0.5),
        Vec3::new(data::TOWER_WIDTH, data::TOWER_HEIGHT, data::TOWER_WIDTH),
    );
    commands.set(tower, MaterialRef::new(material));
    commands.set(tower, ChildOf(root));
}

fn spawn_beacons(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let material = insert_material(
        registry,
        "orrery_beacon",
        Material {
            base_color: data::BEACON_COLOR,
            emissive_factor: data::BEACON_EMISSIVE,
            roughness: 0.4,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        "orrery_beacon_found",
        Material {
            base_color: data::BEACON_COLOR,
            emissive_factor: data::BEACON_FOUND_EMISSIVE,
            roughness: 0.4,
            ..Default::default()
        },
    );

    for definition in data::BEACONS {
        let anchor = Vec3::new(
            definition.anchor[0],
            definition.anchor[1],
            definition.anchor[2],
        );
        let position = queries::surface_position(
            definition.body,
            anchor,
            data::BEACON_HEIGHT * 0.5,
            definition.inside,
            0.0,
        );
        let entity = spawn_mesh_at(
            commands,
            "Cone",
            position,
            Vec3::new(1.3, data::BEACON_HEIGHT, 1.3),
        );
        commands.set(entity, MaterialRef::new(material));
        commands.set(entity, StandingOn(definition.body));
        commands.set(entity, Anchor(anchor));
        commands.set(entity, Inside(definition.inside));
        commands.set(entity, Found(false));
    }
}

fn spawn_ship(home: usize, registry: &mut MaterialRegistry, commands: &mut Commands) -> Entity {
    let (offset, velocity, rotation) = ship_start(home);
    let position = queries::body_position(home, 0.0) + offset;
    let transform = LocalTransform {
        translation: position,
        rotation,
        scale: Vec3::repeat(1.0),
    };
    let root = commands.spawn((
        Name("Ship".to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        Velocity(velocity),
        Landed(None),
        Anchor(Vec3::zeros()),
        Target(home),
        Autopilot(false),
    ));

    let hull_material = insert_material(
        registry,
        "orrery_ship",
        Material {
            base_color: [0.82, 0.84, 0.88, 1.0],
            emissive_factor: [0.04, 0.07, 0.1],
            roughness: 0.35,
            metallic: 0.6,
            ..Default::default()
        },
    );
    let trim_material = insert_material(
        registry,
        "orrery_ship_trim",
        Material {
            base_color: [0.86, 0.36, 0.2, 1.0],
            emissive_factor: [0.16, 0.04, 0.0],
            roughness: 0.5,
            metallic: 0.2,
            ..Default::default()
        },
    );

    let hull = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::zeros(),
        Vec3::new(data::SHIP_HULL[0], data::SHIP_HULL[1], data::SHIP_HULL[2]),
    );
    commands.set(hull, MaterialRef::new(hull_material));
    commands.set(hull, ChildOf(root));

    let nose = spawn_mesh_at(
        commands,
        "Cone",
        Vec3::new(
            0.0,
            0.0,
            -data::SHIP_HULL[2] * 0.5 - data::SHIP_NOSE[1] * 0.5,
        ),
        Vec3::new(data::SHIP_NOSE[0], data::SHIP_NOSE[1], data::SHIP_NOSE[2]),
    );
    commands.set(nose, MaterialRef::new(trim_material));
    commands.set(nose, ChildOf(root));
    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(nose) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());
        }
    });

    for side in [-1.0_f32, 1.0] {
        let wing = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(
                side * data::SHIP_WING[0] * 0.5,
                0.0,
                data::SHIP_HULL[2] * 0.28,
            ),
            Vec3::new(data::SHIP_WING[0], data::SHIP_WING[1], data::SHIP_WING[2]),
        );
        commands.set(wing, MaterialRef::new(hull_material));
        commands.set(wing, ChildOf(root));
    }

    root
}

fn spawn_plume(commands: &mut Commands) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Point,
            spawn_rate: 0.0,
            enabled: false,
            particle_lifetime_min: 0.18,
            particle_lifetime_max: 0.45,
            initial_velocity_min: 2.0,
            initial_velocity_max: 6.0,
            velocity_spread: 0.35,
            gravity: Vec3::zeros(),
            drag: 1.4,
            size_start: 0.55,
            size_end: 0.05,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(0.85, 0.95, 1.0, 1.0)),
                    (0.45, Vec4::new(0.35, 0.7, 1.0, 0.8)),
                    (1.0, Vec4::new(0.1, 0.2, 0.5, 0.0)),
                ],
            },
            emissive_strength: 3.5,
            turbulence_strength: 0.4,
            turbulence_frequency: 2.0,
            ..Default::default()
        },
    );
    entity
}

fn spawn_walker(home: usize, registry: &mut MaterialRegistry, commands: &mut Commands) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        "Cylinder",
        queries::body_position(home, 0.0),
        Vec3::new(
            data::WALKER_SCALE[0],
            data::WALKER_SCALE[1],
            data::WALKER_SCALE[2],
        ),
    );
    let material = insert_material(
        registry,
        "orrery_walker",
        Material {
            base_color: [0.95, 0.62, 0.3, 1.0],
            emissive_factor: [0.2, 0.09, 0.0],
            roughness: 0.7,
            ..Default::default()
        },
    );
    commands.set(entity, MaterialRef::new(material));
    commands.set(entity, StandingOn(home));
    commands.set(
        entity,
        Offset(Vec3::y() * (data::BODIES[home].radius + data::WALKER_HEIGHT)),
    );
    commands.set(entity, Velocity(Vec3::zeros()));
    commands.set(entity, Grounded(false));
    commands.set(entity, Visibility { visible: false });
    entity
}

fn spawn_chase_camera(home: usize, commands: &mut Commands) -> Entity {
    let (offset, _, rotation) = ship_start(home);
    let ship = queries::body_position(home, 0.0) + offset;
    let back = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::z());
    let up = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::y());
    let position = ship + back * data::CHASE_DISTANCE + up * data::CHASE_HEIGHT;
    let camera = spawn_camera(commands, position, "Chase Camera".to_string());
    let facing = queries::basis_rotation(ship - position, up);
    commands.queue(move |storage, resources| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
            transform.rotation = facing;
        }
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
    camera
}
