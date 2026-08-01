use nightshade::prelude::nalgebra_glm;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{EntityCaches, despawn_recursive_now};
use nightshade_graphics::prelude::*;
use nightshade_openxr::OpenXrPlugin;
use nightshade_openxr::contract::{XrFrame, XrHand, XrInput, XrRig, XrRigSlot};
use nightshade_physics::PhysicsPlugin;
use nightshade_physics::contract::{ColliderComponent, RigidBodyComponent};
use nightshade_platform::prelude::*;
use nightshade_render::commands::material_registry::queue_set_material;
use nightshade_render::data::material::Material;

const SKYBOX: &[u8] = include_bytes!("../../../../../assets/sky/winter_river_1k.hdr");
const HELMET: &[u8] = include_bytes!("../../../../../assets/gltf/DamagedHelmet.glb");
const BOULDER: &[u8] = include_bytes!("../../../../../assets/gltf/namaqualand_boulder_02_1k.glb");

const EYE_HEIGHT: f32 = 1.6;
const TABLE_HEIGHT_RATIO: f32 = 0.62;
const TABLE_DISTANCE_RATIO: f32 = 0.34;
const TABLE_THICKNESS: f32 = 0.05;
const TABLE_WIDTH: f32 = 1.2;
const TABLE_DEPTH: f32 = 0.6;
const MINIMUM_EYE_HEIGHT: f32 = 0.3;
const FIREWORK_INTERVAL: f32 = 1.4;
const FIREWORK_LIFETIME: f32 = 4.0;
const PROP_SIZE: f32 = 0.09;
const REACH: f32 = 0.22;
const PEDESTAL_SURFACE: f32 = 0.9;
const PEDESTAL_SPHERE_RADIUS: f32 = 0.16;

const MOVE_SPEED: f32 = 2.0;
const SNAP_TURN_RADIANS: f32 = std::f32::consts::FRAC_PI_6;
const STICK_DEAD_ZONE: f32 = 0.2;
const PEDESTAL_COUNT: usize = 7;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugin(nightshade_window::WindowPlugin::default())
        .add_plugin(nightshade_log::LogPlugin::default())
        .add_plugin(nightshade_wgpu::RenderPlugin::default())
        .add_plugin(nightshade_camera::CameraPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(OpenXrPlugin::default())
        .add_plugin(HelloVrPlugin)
        .run()
}

#[derive(Default)]
struct HelloVr {
    landmarks_asked: Option<nightshade_load::LoadHandle>,
    helmet_asked: Option<nightshade_load::LoadHandle>,
    helmet: Option<Entity>,
    tips: Vec<(XrHand, Entity)>,
    props: Vec<Entity>,
    furniture_placed: bool,
    eye_height: f32,
    fireworks: Vec<(Entity, f32)>,
    launches: u32,
    next_launch: f32,
    held_left: Option<Held>,
    held_right: Option<Held>,
    turn_armed: bool,
}

#[derive(Clone, Copy)]
struct Held {
    entity: Entity,
    offset_position: Vec3,
    offset_rotation: Quat,
}

struct HelloVrPlugin;

impl Plugin for HelloVrPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "hello_vr".to_string();
        app.insert_resource(HelloVr::default());
        app.add_system(Stage::Startup, initialize);
        app.add_system(
            Stage::Update,
            (
                stand_the_scenery,
                place_furniture,
                spin_helmet,
                drive_wands,
                grab_props,
                carry_props,
                locomotion,
                fireworks,
            ),
        );
    }
}

fn initialize(
    mut hello: ResMut<HelloVr>,
    mut debug_draw: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut queues: ResMut<nightshade_graphics::resources::queues::CommandQueues>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    rig: Res<XrRigSlot>,
    mut commands: Commands,
) {
    debug_draw.show_grid = false;
    settings.atmosphere = Atmosphere::Hdr;
    settings.ssao_enabled = true;
    settings.ssao_intensity = 1.2;
    settings.ssao_radius = 0.4;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.25;
    load_hdr_skybox(&mut queues, SKYBOX.to_vec());

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.cast_shadows = true;
            light.intensity = 2.5;
        }
    });

    spawn_floor(&mut commands);
    spawn_pedestals(&mut commands);
    hello.landmarks_asked = Some(nightshade_load::load(
        &mut loading,
        nightshade_load::Source::Bytes(BOULDER.to_vec()),
    ));
    hello.helmet_asked = Some(nightshade_load::load(
        &mut loading,
        nightshade_load::Source::Bytes(HELMET.to_vec()),
    ));

    if let Some(rig) = rig.0 {
        for hand in XrHand::ALL {
            let tip = spawn_wand(&mut commands, rig.hand(hand), hand);
            hello.tips.push((hand, tip));
        }
    }
}

fn spawn_floor(commands: &mut Commands) {
    let floor = spawn_plane_at(commands, Vec3::zeros());
    commands.set(
        floor,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(80.0, 1.0, 80.0),
        },
    );
    queue_set_material(
        commands,
        floor,
        "floor",
        Material {
            base_color: [0.03, 0.035, 0.045, 1.0],
            roughness: 0.18,
            metallic: 0.9,
            ..Default::default()
        },
    );
    commands.set(
        floor,
        RigidBodyComponent::new_static().with_translation(0.0, 0.0, 0.0),
    );
    commands.set(floor, ColliderComponent::new_cuboid(40.0, 0.05, 40.0));
}

fn spawn_pedestals(commands: &mut Commands) {
    let arc = std::f32::consts::PI * 1.35;
    for index in 0..PEDESTAL_COUNT {
        let fraction = index as f32 / (PEDESTAL_COUNT - 1) as f32;
        let angle = -arc * 0.5 + arc * fraction - std::f32::consts::FRAC_PI_2;

        let radius = if index % 2 == 0 { 2.4 } else { 3.6 };
        let position = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

        let pedestal_at = position + Vec3::new(0.0, PEDESTAL_SURFACE * 0.5, 0.0);
        let pedestal = spawn_cylinder_at(commands, pedestal_at);
        commands.set(
            pedestal,
            LocalTransform {
                translation: pedestal_at,
                rotation: Quat::identity(),
                scale: Vec3::new(0.22, PEDESTAL_SURFACE, 0.22),
            },
        );
        queue_set_material(
            commands,
            pedestal,
            format!("pedestal_{index}"),
            Material {
                base_color: [0.16, 0.17, 0.19, 1.0],
                roughness: 0.75,
                ..Default::default()
            },
        );
        commands.set(pedestal, CastsShadow);

        let sphere_at = position + Vec3::new(0.0, PEDESTAL_SURFACE + PEDESTAL_SPHERE_RADIUS, 0.0);
        let sphere = spawn_sphere_at(commands, sphere_at);
        commands.set(
            sphere,
            LocalTransform {
                translation: sphere_at,
                rotation: Quat::identity(),
                scale: Vec3::new(
                    PEDESTAL_SPHERE_RADIUS,
                    PEDESTAL_SPHERE_RADIUS,
                    PEDESTAL_SPHERE_RADIUS,
                ),
            },
        );
        queue_set_material(
            commands,
            sphere,
            format!("sphere_{index}"),
            Material {
                base_color: [
                    0.95 - 0.6 * fraction,
                    0.55 + 0.15 * fraction,
                    0.35 + 0.5 * fraction,
                    1.0,
                ],
                roughness: (0.9 - fraction * 0.85).max(0.05),
                metallic: fraction,
                ..Default::default()
            },
        );
        commands.set(sphere, CastsShadow);
    }
}

fn stand_the_scenery(
    mut hello: ResMut<HelloVr>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    if let Some(handle) = hello.landmarks_asked
        && let Some(outcome) = nightshade_load::take(&mut loading, handle)
    {
        hello.landmarks_asked = None;
        spawn_landmarks(storage, parts.reborrow(), outcome);
    }
    if let Some(handle) = hello.helmet_asked
        && let Some(outcome) = nightshade_load::take(&mut loading, handle)
    {
        hello.helmet_asked = None;
        hello.helmet = spawn_helmet(storage, parts.reborrow(), outcome);
    }
}

fn spawn_landmarks(
    storage: &mut nightshade::ecs::Storage,
    mut parts: nightshade_prefab::PrefabParts<'_>,
    outcome: Result<nightshade_prefab::data::ModelImport, String>,
) {
    let Ok(mut import) = outcome else {
        tracing::error!("failed to load the landmark model");
        return;
    };
    nightshade_prefab::queue_model_load(
        &mut nightshade_prefab::PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        },
        &mut import,
    );
    let prefabs = import.prefabs;
    if prefabs.is_empty() {
        return;
    }

    for ring in 0..2 {
        let radius = 9.0 + ring as f32 * 12.0;
        let count = 10 + ring * 8;
        for index in 0..count {
            let fraction = index as f32 / count as f32;
            let angle = fraction * std::f32::consts::TAU + ring as f32 * 0.7;
            let jitter = ((index * 7 + ring * 13) % 11) as f32 / 11.0;
            let distance = radius + jitter * 6.0;
            let position = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let entity = nightshade_prefab::spawn_prefab(
                &mut nightshade_prefab::PrefabSpawn {
                    storage,
                    parts: parts.reborrow(),
                },
                &prefabs[index % prefabs.len()],
                position,
            );
            if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
                let scale = 0.9 + jitter * 1.6;
                transform.scale = Vec3::new(scale, scale, scale);
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(fraction * std::f32::consts::TAU, &Vec3::y());
            }
            storage.set(entity, CastsShadow);
        }
    }
}

fn place_furniture(mut hello: ResMut<HelloVr>, xr_frame: Res<XrFrame>, mut commands: Commands) {
    if hello.furniture_placed {
        return;
    }
    if !xr_frame.tracking_valid {
        return;
    }
    let eye_height = xr_frame.head_position.y;
    if eye_height <= MINIMUM_EYE_HEIGHT {
        return;
    }

    hello.eye_height = eye_height;
    let surface = eye_height * TABLE_HEIGHT_RATIO;
    let distance = eye_height * TABLE_DISTANCE_RATIO;
    spawn_table(&mut commands, surface, distance);
    spawn_props(&mut commands, &mut hello, surface, distance);
    spawn_height_reference(&mut commands, eye_height, distance);
    hello.furniture_placed = true;
}

fn fireworks(
    mut hello: ResMut<HelloVr>,
    time: Res<Time>,
    mut caches: EntityCaches,
    storage: &mut nightshade::ecs::Storage,
) {
    if hello.eye_height < MINIMUM_EYE_HEIGHT {
        return;
    }
    let delta_time = time.delta_time;

    hello.fireworks.retain_mut(|(entity, remaining)| {
        *remaining -= delta_time;
        if *remaining > 0.0 {
            return true;
        }
        despawn_recursive_now(storage, &mut caches, *entity);
        false
    });

    hello.next_launch -= delta_time;
    if hello.next_launch > 0.0 {
        return;
    }
    hello.next_launch = FIREWORK_INTERVAL;

    let launch = hello.launches;
    hello.launches = launch.wrapping_add(1);

    let angle = launch as f32 * 2.399;
    let distance = hello.eye_height * (9.0 + ((launch % 5) as f32) * 1.6);
    let height = hello.eye_height * (4.5 + ((launch % 3) as f32) * 1.4);
    let position = Vec3::new(angle.cos() * distance, height, angle.sin() * distance);

    let palette = [
        Vec3::new(1.0, 0.35, 0.25),
        Vec3::new(0.35, 0.75, 1.0),
        Vec3::new(1.0, 0.85, 0.35),
        Vec3::new(0.65, 0.45, 1.0),
        Vec3::new(0.45, 1.0, 0.6),
    ];
    let color = palette[launch as usize % palette.len()];

    let entity = storage.spawn((LocalTransform::default(), GlobalTransform::default()));
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.translation = position;
    }
    storage.set(
        entity,
        nightshade_particles::contract::ParticleEmitter::firework_explosion(position, color, 160),
    );
    hello.fireworks.push((entity, FIREWORK_LIFETIME));
}

fn spawn_height_reference(commands: &mut Commands, eye_height: f32, distance: f32) {
    let post_at = Vec3::new(TABLE_WIDTH * 0.75, eye_height * 0.5, -distance);
    let post = spawn_cylinder_at(commands, post_at);
    commands.set(
        post,
        LocalTransform {
            translation: post_at,
            rotation: Quat::identity(),
            scale: Vec3::new(0.04, eye_height, 0.04),
        },
    );
    queue_set_material(
        commands,
        post,
        "height_reference",
        Material {
            base_color: [0.95, 0.95, 0.95, 1.0],
            emissive_factor: [0.35, 0.35, 0.4],
            roughness: 0.5,
            ..Default::default()
        },
    );
    commands.set(post, CastsShadow);

    let cap_at = Vec3::new(TABLE_WIDTH * 0.75, eye_height, -distance);
    let cap = spawn_cylinder_at(commands, cap_at);
    commands.set(
        cap,
        LocalTransform {
            translation: cap_at,
            rotation: Quat::identity(),
            scale: Vec3::new(0.12, 0.02, 0.12),
        },
    );
    queue_set_material(
        commands,
        cap,
        "height_reference_cap",
        Material {
            base_color: [0.95, 0.3, 0.25, 1.0],
            emissive_factor: [1.2, 0.2, 0.15],
            roughness: 0.4,
            ..Default::default()
        },
    );
}

fn spawn_table(commands: &mut Commands, surface: f32, distance: f32) {
    let underside = surface - TABLE_THICKNESS;
    let slab_center = surface - TABLE_THICKNESS * 0.5;

    let top_at = Vec3::new(0.0, slab_center, -distance);
    let top = spawn_cube_at(commands, top_at);
    commands.set(
        top,
        LocalTransform {
            translation: top_at,
            rotation: Quat::identity(),
            scale: Vec3::new(TABLE_WIDTH, TABLE_THICKNESS, TABLE_DEPTH),
        },
    );
    queue_set_material(
        commands,
        top,
        "table_top",
        Material {
            base_color: [0.32, 0.22, 0.14, 1.0],
            roughness: 0.55,
            ..Default::default()
        },
    );
    commands.set(top, CastsShadow);
    commands.set(
        top,
        RigidBodyComponent::new_static().with_translation(0.0, slab_center, -distance),
    );
    commands.set(
        top,
        ColliderComponent::new_cuboid(TABLE_WIDTH * 0.5, TABLE_THICKNESS * 0.5, TABLE_DEPTH * 0.5),
    );

    let inset = 0.08;
    let corners = [
        (-TABLE_WIDTH * 0.5 + inset, -TABLE_DEPTH * 0.5 + inset),
        (TABLE_WIDTH * 0.5 - inset, -TABLE_DEPTH * 0.5 + inset),
        (-TABLE_WIDTH * 0.5 + inset, TABLE_DEPTH * 0.5 - inset),
        (TABLE_WIDTH * 0.5 - inset, TABLE_DEPTH * 0.5 - inset),
    ];
    for (index, (x, z)) in corners.into_iter().enumerate() {
        let leg_at = Vec3::new(x, underside * 0.5, -distance + z);
        let leg = spawn_cylinder_at(commands, leg_at);
        commands.set(
            leg,
            LocalTransform {
                translation: leg_at,
                rotation: Quat::identity(),
                scale: Vec3::new(0.05, underside, 0.05),
            },
        );
        queue_set_material(
            commands,
            leg,
            format!("table_leg_{index}"),
            Material {
                base_color: [0.12, 0.12, 0.13, 1.0],
                roughness: 0.4,
                metallic: 0.8,
                ..Default::default()
            },
        );
        commands.set(leg, CastsShadow);
    }
}

fn spawn_props(commands: &mut Commands, hello: &mut HelloVr, surface: f32, distance: f32) {
    let palette = [
        [0.90, 0.25, 0.22],
        [0.95, 0.70, 0.20],
        [0.30, 0.75, 0.40],
        [0.25, 0.55, 0.92],
        [0.70, 0.35, 0.85],
        [0.95, 0.95, 0.95],
    ];
    let half = PROP_SIZE * 0.5;
    let columns = 3;

    for (index, color) in palette.into_iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        let spacing = 0.22;
        let position = Vec3::new(
            (column as f32 - (columns - 1) as f32 * 0.5) * spacing,
            surface + half,
            -distance + (row as f32 - 0.5) * spacing,
        );

        let is_cube = index % 2 == 0;
        let entity = if is_cube {
            spawn_cube_at(commands, position)
        } else {
            spawn_sphere_at(commands, position)
        };
        let scale = if is_cube { PROP_SIZE } else { half };
        commands.set(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(scale, scale, scale),
            },
        );
        queue_set_material(
            commands,
            entity,
            format!("prop_{index}"),
            Material {
                base_color: [color[0], color[1], color[2], 1.0],
                roughness: 0.3,
                metallic: if index % 3 == 0 { 0.8 } else { 0.0 },
                ..Default::default()
            },
        );
        commands.set(entity, CastsShadow);
        commands.set(
            entity,
            RigidBodyComponent::new_dynamic().with_translation(position.x, position.y, position.z),
        );
        commands.set(
            entity,
            if is_cube {
                ColliderComponent::new_cuboid(half, half, half)
            } else {
                ColliderComponent::new_ball(half)
            },
        );
        hello.props.push(entity);
    }
}

fn spawn_helmet(
    storage: &mut nightshade::ecs::Storage,
    parts: nightshade_prefab::PrefabParts<'_>,
    outcome: Result<nightshade_prefab::data::ModelImport, String>,
) -> Option<Entity> {
    let mut import = outcome
        .inspect_err(|error| tracing::error!("failed to load the helmet: {error}"))
        .ok()?;
    let mut spawn = nightshade_prefab::PrefabSpawn { storage, parts };
    nightshade_prefab::queue_model_load(&mut spawn, &mut import);
    let root = import.prefabs.first().map(|prefab| {
        nightshade_prefab::spawn_prefab(&mut spawn, prefab, Vec3::new(0.0, EYE_HEIGHT, -1.6))
    })?;
    if let Some(transform) = storage.get_mut::<LocalTransform>(root) {
        transform.scale = Vec3::new(0.55, 0.55, 0.55);
    }
    Some(root)
}

fn spawn_wand(commands: &mut Commands, hand_entity: Entity, hand: XrHand) -> Entity {
    let tint = hand_tint(hand);

    let shaft = spawn_cylinder_at(commands, Vec3::zeros());
    commands.set(shaft, nightshade::ecs::ChildOf(hand_entity));
    commands.set(
        shaft,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, -0.06),
            rotation: nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(0.018, 0.075, 0.018),
        },
    );
    queue_set_material(
        commands,
        shaft,
        format!("wand_shaft_{hand:?}"),
        Material {
            base_color: [0.1, 0.1, 0.12, 1.0],
            roughness: 0.4,
            metallic: 0.8,
            ..Default::default()
        },
    );

    let tip = spawn_sphere_at(commands, Vec3::zeros());
    commands.set(tip, nightshade::ecs::ChildOf(hand_entity));
    commands.set(
        tip,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, -0.14),
            rotation: Quat::identity(),
            scale: Vec3::new(0.03, 0.03, 0.03),
        },
    );
    queue_set_material(
        commands,
        tip,
        format!("wand_tip_{hand:?}"),
        Material {
            base_color: [tint[0], tint[1], tint[2], 1.0],
            emissive_factor: tint,
            roughness: 0.3,
            ..Default::default()
        },
    );
    tip
}

fn hand_tint(hand: XrHand) -> [f32; 3] {
    match hand {
        XrHand::Left => [0.15, 0.75, 0.95],
        XrHand::Right => [0.98, 0.62, 0.15],
    }
}

fn spin_helmet(hello: Res<HelloVr>, time: Res<Time>, storage: &mut nightshade::ecs::Storage) {
    let delta_time = time.delta_time;
    let Some(helmet) = hello.helmet else {
        return;
    };
    if let Some(transform) = storage.get_mut::<LocalTransform>(helmet) {
        transform.rotation =
            nalgebra_glm::quat_angle_axis(delta_time * 0.35, &Vec3::y()) * transform.rotation;
    }
}

fn drive_wands(hello: Res<HelloVr>, input: Res<XrInput>, mut commands: Commands) {
    let input = *input;
    for (hand, tip) in hello.tips.clone() {
        let state = XrRig::hand_state(&input, hand);
        let glow = 0.4 + state.grip.max(state.trigger) * 6.0;
        let tint = hand_tint(hand);
        queue_set_material(
            &mut commands,
            tip,
            format!("wand_tip_{hand:?}"),
            Material {
                base_color: [tint[0], tint[1], tint[2], 1.0],
                emissive_factor: [tint[0] * glow, tint[1] * glow, tint[2] * glow],
                roughness: 0.3,
                ..Default::default()
            },
        );
    }
}

fn grab_props(
    mut hello: ResMut<HelloVr>,
    input: Res<XrInput>,
    rig: Res<XrRigSlot>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(rig) = rig.0 else {
        return;
    };
    let input = *input;

    for hand in XrHand::ALL {
        let squeezing = XrRig::hand_state(&input, hand).grip_pressed();
        let hand_entity = rig.hand(hand);
        let tip = hello
            .tips
            .iter()
            .find(|(side, _)| *side == hand)
            .map(|(_, tip)| *tip)
            .unwrap_or(hand_entity);
        let held = match hand {
            XrHand::Left => hello.held_left,
            XrHand::Right => hello.held_right,
        };

        let next = match (squeezing, held) {
            (true, None) => nearest_prop(storage, &hello.props, tip)
                .and_then(|entity| begin_hold(storage, entity, hand_entity, rig.origin)),
            (false, Some(held)) => {
                release(storage, held.entity);
                None
            }
            (_, current) => current,
        };

        match hand {
            XrHand::Left => hello.held_left = next,
            XrHand::Right => hello.held_right = next,
        }
    }
}

fn begin_hold(
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    hand_entity: Entity,
    origin: Entity,
) -> Option<Held> {
    let (hand_position, hand_rotation) = world_pose(storage, hand_entity, origin)?;
    let transform = storage.get::<LocalTransform>(entity)?;
    let prop_position = transform.translation;
    let prop_rotation = transform.rotation;

    storage.remove::<RigidBodyComponent>(entity);

    let into_hand = nalgebra_glm::quat_conjugate(&hand_rotation);
    Some(Held {
        entity,
        offset_position: nalgebra_glm::quat_rotate_vec3(
            &into_hand,
            &(prop_position - hand_position),
        ),
        offset_rotation: into_hand * prop_rotation,
    })
}

fn carry_props(hello: Res<HelloVr>, rig: Res<XrRigSlot>, storage: &mut nightshade::ecs::Storage) {
    let Some(rig) = rig.0 else {
        return;
    };
    for (hand, held) in [
        (XrHand::Left, hello.held_left),
        (XrHand::Right, hello.held_right),
    ] {
        let Some(held) = held else { continue };
        let Some((hand_position, hand_rotation)) = world_pose(storage, rig.hand(hand), rig.origin)
        else {
            continue;
        };
        let position =
            hand_position + nalgebra_glm::quat_rotate_vec3(&hand_rotation, &held.offset_position);
        let rotation = hand_rotation * held.offset_rotation;
        if let Some(transform) = storage.get_mut::<LocalTransform>(held.entity) {
            transform.translation = position;
            transform.rotation = rotation;
        }
    }
}

fn release(storage: &mut nightshade::ecs::Storage, entity: Entity) {
    let position = storage
        .get::<LocalTransform>(entity)
        .map(|transform| transform.translation)
        .unwrap_or_else(Vec3::zeros);
    storage.set(
        entity,
        RigidBodyComponent::new_dynamic().with_translation(position.x, position.y, position.z),
    );
}

fn world_pose(
    storage: &nightshade::ecs::Storage,
    entity: Entity,
    origin: Entity,
) -> Option<(Vec3, Quat)> {
    let position = storage.get::<GlobalTransform>(entity)?.translation();
    let origin_rotation = storage
        .get::<LocalTransform>(origin)
        .map(|transform| transform.rotation)
        .unwrap_or_else(Quat::identity);
    let local_rotation = storage.get::<LocalTransform>(entity)?.rotation;
    Some((position, origin_rotation * local_rotation))
}

fn nearest_prop(
    storage: &nightshade::ecs::Storage,
    props: &[Entity],
    tip: Entity,
) -> Option<Entity> {
    let tip_position = storage.get::<GlobalTransform>(tip)?.translation();
    props
        .iter()
        .copied()
        .filter_map(|entity| {
            let position = storage.get::<LocalTransform>(entity)?.translation;
            let distance = (position - tip_position).norm();
            (distance < REACH).then_some((entity, distance))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(entity, _)| entity)
}

fn locomotion(
    mut hello: ResMut<HelloVr>,
    input: Res<XrInput>,
    rig: Res<XrRigSlot>,
    xr_frame: Res<XrFrame>,
    time: Res<Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(rig) = rig.0 else {
        return;
    };
    let input = *input;
    let delta_time = time.delta_time;

    let head_yaw = yaw_of(&xr_frame.head_orientation);
    let origin_yaw = storage
        .get::<LocalTransform>(rig.origin)
        .map(|transform| yaw_of(&transform.rotation))
        .unwrap_or(0.0);

    let stick = input.left.thumbstick;
    let mut translation = Vec3::zeros();
    if stick.norm() > STICK_DEAD_ZONE {
        let yaw = origin_yaw + head_yaw;
        let forward = Vec3::new(yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, yaw.sin());
        translation = (forward * stick.y + right * stick.x) * MOVE_SPEED * delta_time;
    }

    let turn_axis = input.right.thumbstick.x;
    let mut turn = 0.0;
    if turn_axis.abs() > 0.6 {
        if !hello.turn_armed {
            turn = -turn_axis.signum() * SNAP_TURN_RADIANS;
            hello.turn_armed = true;
        }
    } else if turn_axis.abs() < 0.3 {
        hello.turn_armed = false;
    }

    if translation == Vec3::zeros() && turn == 0.0 {
        return;
    }
    if let Some(transform) = storage.get_mut::<LocalTransform>(rig.origin) {
        transform.translation += translation;
        if turn != 0.0 {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(turn, &Vec3::y()) * transform.rotation;
        }
    }
}

fn yaw_of(rotation: &Quat) -> f32 {
    let forward = nalgebra_glm::quat_rotate_vec3(rotation, &Vec3::new(0.0, 0.0, -1.0));
    forward.x.atan2(-forward.z)
}
