use crate::components::{
    Around, Autopilot, BodyRadius, Bulk, Footing, Fuel, Grip, Ground, Grounded, Heading, Locked,
    Motion, Offset, Pitch, Pull, Push, Reach, SpinRate, Velocity,
};
use crate::data::Settings;
use crate::data::mode::Event as ModeEvent;
use crate::data::{
    BALL_MESH, BALL_SEGMENTS, BEACON_GLOW, BEACON_GLOW_STRENGTH, BODIES, BodyPlan, EXIT_OFFSET,
    FIELD_OF_VIEW_DEGREES, HEADLIGHT, HEADLIGHT_COLOR, HEADLIGHT_INNER, HEADLIGHT_OUTER,
    HEADLIGHT_RANGE, HEADLIGHTS, HOME, HULL, HULL_COLOR, HULL_GLOW, HULL_GLOW_STRENGTH, HULL_SHEEN,
    PAD_DIRECTION, REACH_FACTOR, RING_COLOR, RING_SPREAD, RING_THICKNESS, SHIP_RADIUS, STARLIGHT,
    STARLIGHT_COLOR, SUN_COLOR, SUN_DAY, SUN_GLOW, SUN_GLOW_STRENGTH, SUN_GRAVITY, SUN_LIGHT,
    SUN_LIGHT_COLOR, SUN_RADIUS, WALKER_RADIUS,
};
use crate::queries::{level_on, orbit_point, orbit_velocity};
use crate::resources::{Arguments, Screen, TidefallResources};
use crate::systems::surface::scatter;
use crate::systems::{hud, screens};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::create_sphere_mesh;
use nightshade_ui::components::UiCanvasData;
use nightshade_ui::prelude::*;
use nightshade_ui_config::prelude::ConfigPanel;
use nightshade_ui_menu::prelude::{MenuStyle, Menus};
use std::f32::consts::TAU;

pub fn initialize(
    mut resources: ResMut<TidefallResources>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut meshes: ResMut<MeshCache>,
    mut window: ResMut<Window>,
    arguments: Res<Arguments>,
    mut menus: ResMut<Menus<Screen>>,
    mut rows: ResMut<ConfigPanel<Settings>>,
    style: Res<MenuStyle>,
    config: Res<Config<Settings>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Space;
    settings.bloom_enabled = true;
    set_cursor_visible(&mut window, true);

    mesh_cache_insert(
        &mut meshes,
        BALL_MESH.to_string(),
        create_sphere_mesh(1.0, BALL_SEGMENTS),
    );
    mesh_cache_protect(&mut meshes, BALL_MESH.to_string());

    let sun = spawn_the_sun(&mut registry, &mut commands);
    let bodies = spawn_the_bodies(&mut registry, &mut commands, sun);

    let beacon_glow = insert_material(
        &mut registry,
        "tidefall_beacon",
        Material {
            base_color: [0.8, 0.95, 1.0, 1.0],
            emissive_factor: BEACON_GLOW,
            emissive_strength: BEACON_GLOW_STRENGTH,
            unlit: true,
            ..Default::default()
        },
    );
    let pad = PAD_DIRECTION.normalize();
    for (index, (plan, (entity, _))) in BODIES.iter().zip(bodies.iter()).enumerate() {
        scatter(
            &mut commands,
            &mut registry,
            &mut resources.roll,
            MaterialRef::new(beacon_glow),
            *entity,
            plan,
            index,
            (index == HOME).then_some(pad),
        );
    }

    let home = &BODIES[HOME];
    let tilt = nalgebra_glm::quat_angle_axis(home.tilt, &Vec3::x());
    let berth = pad * (home.radius + SHIP_RADIUS);
    let heading = level_on(pad, Vec3::new(0.0, 0.0, -1.0));
    let parked = bodies[HOME].1 + nalgebra_glm::quat_rotate_vec3(&tilt, &berth);
    let stance = nalgebra_glm::quat_normalize(&(tilt * heading));
    let ship = spawn_the_ship(
        &mut registry,
        &mut commands,
        bodies[HOME].0,
        parked,
        stance,
        berth,
        heading,
    );
    resources.ship = Some(ship);
    resources.headlights = HEADLIGHTS
        .iter()
        .map(|offset| {
            let lamp = commands.spawn((
                Name("Headlight".to_string()),
                LocalTransform::from_translation(*offset),
                GlobalTransform::default(),
                Light {
                    light_type: LightType::Spot,
                    color: HEADLIGHT_COLOR,
                    intensity: HEADLIGHT,
                    range: HEADLIGHT_RANGE,
                    inner_cone_angle: HEADLIGHT_INNER,
                    outer_cone_angle: HEADLIGHT_OUTER,
                    cast_shadows: false,
                    ..Default::default()
                },
            ));
            commands.set(lamp, ChildOf(ship));
            lamp
        })
        .collect();
    if arguments.aboard
        && let Some(next) = resources.mode.process_event(ModeEvent::Board)
    {
        resources.mode = next;
    }
    if let Some(wanted) = arguments.target.as_deref() {
        let wanted = wanted.to_lowercase();
        let target = BODIES
            .iter()
            .zip(bodies.iter())
            .find(|(plan, _)| plan.name.to_lowercase() == wanted)
            .map(|(_, (entity, _))| *entity);
        commands.set(ship, Locked(target));
    }
    resources.walker = Some(spawn_the_walker(&mut commands, parked, stance));

    let mut tree = UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    let (screen_handles, built) = screens::build(&mut tree, &style, &mut menus, &config);
    tree.finish();
    resources.screens = screen_handles;
    *rows = built;
    commands.set(resources.hud.canvas, UiCanvasData::default());
}

fn spawn_ball(
    commands: &mut Commands,
    name: &str,
    center: Vec3,
    radius: f32,
    tilt: f32,
    material: MaterialRef,
) -> Entity {
    let transform = LocalTransform {
        translation: center,
        rotation: nalgebra_glm::quat_angle_axis(tilt, &Vec3::x()),
        scale: Vec3::repeat(radius),
    };
    let entity = commands.spawn((
        Name(name.to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(BALL_MESH),
        material,
        Visibility { visible: true },
        BoundingVolume::from_mesh_type("Sphere"),
        CastsShadow,
    ));
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

fn spawn_the_sun(registry: &mut MaterialRegistry, commands: &mut Commands) -> Entity {
    let material = insert_material(
        registry,
        "tidefall_sun",
        Material {
            base_color: SUN_COLOR,
            emissive_factor: SUN_GLOW,
            emissive_strength: SUN_GLOW_STRENGTH,
            unlit: true,
            ..Default::default()
        },
    );
    let sun = spawn_ball(
        commands,
        "Sun",
        Vec3::zeros(),
        SUN_RADIUS,
        0.0,
        MaterialRef::new(material),
    );
    set_the_path(
        commands,
        sun,
        None,
        Vec3::zeros(),
        Vec3::zeros(),
        TAU / SUN_DAY,
        SUN_GRAVITY * SUN_RADIUS,
    );
    commands.set(sun, BodyRadius(SUN_RADIUS));
    commands.set(sun, Reach(f32::INFINITY));
    commands.set(sun, Grip(1.0));
    commands.spawn((
        Name("Sunlight".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: SUN_LIGHT_COLOR,
            intensity: SUN_LIGHT,
            range: 0.0,
            cast_shadows: false,
            ..Default::default()
        },
    ));
    commands.spawn((
        Name("Starlight".to_string()),
        LocalTransform {
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_3, &Vec3::x()),
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Directional,
            color: STARLIGHT_COLOR,
            intensity: STARLIGHT,
            cast_shadows: false,
            ..Default::default()
        },
    ));
    sun
}

fn spawn_the_bodies(
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
    sun: Entity,
) -> Vec<(Entity, Vec3)> {
    let mut bodies: Vec<(Entity, Vec3)> = Vec::with_capacity(BODIES.len());
    let ring = insert_material(
        registry,
        "tidefall_ring",
        Material {
            base_color: RING_COLOR,
            roughness: 0.9,
            double_sided: true,
            ..Default::default()
        },
    );
    for plan in BODIES.iter() {
        let (primary, primary_center, primary_pull) = match plan.around {
            Some(around) => (
                bodies[around].0,
                bodies[around].1,
                BODIES[around].gravity * BODIES[around].radius,
            ),
            None => (sun, Vec3::zeros(), SUN_GRAVITY * SUN_RADIUS),
        };
        let offset = orbit_point(plan.orbit, plan.phase);
        let center = primary_center + offset;
        let material = insert_material(
            registry,
            format!("tidefall_{}", plan.name.to_lowercase()),
            Material {
                base_color: plan.color,
                roughness: plan.roughness,
                metallic: plan.metallic,
                ..Default::default()
            },
        );
        let entity = spawn_ball(
            commands,
            plan.name,
            center,
            plan.radius,
            plan.tilt,
            MaterialRef::new(material),
        );
        set_the_path(
            commands,
            entity,
            Some(primary),
            offset,
            orbit_velocity(primary_pull.sqrt(), plan.phase),
            TAU / plan.day,
            plan.gravity * plan.radius,
        );
        commands.set(entity, BodyRadius(plan.radius));
        commands.set(entity, Reach(plan.radius * REACH_FACTOR));
        commands.set(entity, Grip(plan.grip));
        if plan.ringed {
            spawn_the_ring(commands, entity, plan, center, MaterialRef::new(ring));
        }
        bodies.push((entity, center));
    }
    bodies
}

fn set_the_path(
    commands: &mut Commands,
    entity: Entity,
    primary: Option<Entity>,
    offset: Vec3,
    velocity: Vec3,
    spin: f32,
    pull: f32,
) {
    commands.set(entity, Around(primary));
    commands.set(entity, Offset(offset));
    commands.set(entity, Velocity(velocity));
    commands.set(entity, SpinRate(spin));
    commands.set(entity, Pull(pull));
    commands.set(entity, Motion(Vec3::zeros()));
}

fn spawn_the_ring(
    commands: &mut Commands,
    body: Entity,
    plan: &BodyPlan,
    center: Vec3,
    material: MaterialRef,
) {
    let spread = plan.radius * RING_SPREAD;
    let entity = spawn_mesh_at(
        commands,
        "Torus",
        center,
        Vec3::new(spread, RING_THICKNESS, spread),
    );
    if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
        transform.rotation = nalgebra_glm::quat_angle_axis(plan.tilt, &Vec3::x());
    }
    commands.restage::<LocalTransform>(entity);
    commands.set(entity, Name(format!("{} ring", plan.name)));
    commands.set(entity, material);
    set_the_path(
        commands,
        entity,
        Some(body),
        Vec3::zeros(),
        Vec3::zeros(),
        TAU / plan.day * 0.5,
        0.0,
    );
}

fn spawn_the_ship(
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
    home: Entity,
    parked: Vec3,
    stance: Quat,
    berth: Vec3,
    heading: Quat,
) -> Entity {
    let transform = LocalTransform {
        translation: parked,
        rotation: stance,
        scale: Vec3::repeat(1.0),
    };
    let ship = commands.spawn((
        Name("Ship".to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        Velocity(Vec3::zeros()),
        Footing::Landed,
        Ground(Some(home)),
        Offset(berth),
        Heading(heading),
        Bulk(SHIP_RADIUS),
        Locked(None),
        Autopilot::default(),
        Push(Vec3::zeros()),
    ));

    let hull = insert_material(
        registry,
        "tidefall_hull",
        Material {
            base_color: HULL_COLOR,
            emissive_factor: HULL_SHEEN,
            emissive_strength: 1.0,
            roughness: 0.45,
            metallic: 0.7,
            ..Default::default()
        },
    );
    let glow = insert_material(
        registry,
        "tidefall_hull_glow",
        Material {
            base_color: [1.0, 0.6, 0.3, 1.0],
            emissive_factor: HULL_GLOW,
            emissive_strength: HULL_GLOW_STRENGTH,
            unlit: true,
            ..Default::default()
        },
    );
    for piece in HULL.iter() {
        let entity = spawn_mesh_at(commands, piece.mesh, piece.offset, piece.scale);
        if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
            transform.rotation = nalgebra_glm::quat_angle_axis(piece.tilt, &Vec3::x());
        }
        commands.restage::<LocalTransform>(entity);
        commands.set(entity, ChildOf(ship));
        commands.set(
            entity,
            MaterialRef::new(if piece.glowing { glow } else { hull }),
        );
    }
    ship
}

fn spawn_the_walker(commands: &mut Commands, parked: Vec3, stance: Quat) -> Entity {
    let right = nalgebra_glm::quat_rotate_vec3(&stance, &Vec3::x());
    let eye = parked - right * EXIT_OFFSET;
    let rotation = nalgebra_glm::quat_normalize(
        &(stance * nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::y())),
    );
    let entity = spawn_camera(commands, eye, "Cockpit".to_string());
    if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
        transform.rotation = rotation;
    }
    commands.restage::<LocalTransform>(entity);
    if let Some(camera) = commands.staged_mut::<Camera>(entity) {
        camera.projection = Projection::Perspective(PerspectiveCamera {
            y_fov_rad: FIELD_OF_VIEW_DEGREES.to_radians(),
            ..Default::default()
        });
    }
    commands.restage::<Camera>(entity);
    commands.set(entity, Velocity(Vec3::zeros()));
    commands.set(entity, Ground(None));
    commands.set(entity, Grounded(false));
    commands.set(entity, Pitch(0.0));
    commands.set(entity, Fuel(1.0));
    commands.set(entity, Bulk(WALKER_RADIUS));
    commands.queue(move |_, resources| {
        resources.res_mut::<ActiveCamera>().0 = Some(entity);
    });
    entity
}
