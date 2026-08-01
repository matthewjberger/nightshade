use crate::data::{Applied, REGION_COUNT, Region, regions_for, threads};
use crate::resources::{BoidsArguments, BoidsResources, GridBuilder, Integration, bounds_for};
use crate::systems::{flock, hud, population};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::UiTreeBuilder;

const FLOCK_MATERIAL: &str = "Flock";

pub fn insert_flock_resources(app: &mut App, starting: usize, regions: usize) {
    app.insert_resource(GridBuilder {
        bounds: bounds_for(starting),
        ..Default::default()
    });
    app.insert_resource(Integration {
        regions,
        ..Default::default()
    });
    app.insert_resource(Applied::default());
    insert_regions(app);
}

fn insert_regions(app: &mut App) {
    app.insert_resource(Region::<0>::default());
    app.insert_resource(Region::<1>::default());
    app.insert_resource(Region::<2>::default());
    app.insert_resource(Region::<3>::default());
    app.insert_resource(Region::<4>::default());
    app.insert_resource(Region::<5>::default());
    app.insert_resource(Region::<6>::default());
    app.insert_resource(Region::<7>::default());
    app.insert_resource(Region::<8>::default());
    app.insert_resource(Region::<9>::default());
    app.insert_resource(Region::<10>::default());
    app.insert_resource(Region::<11>::default());
    app.insert_resource(Region::<12>::default());
    app.insert_resource(Region::<13>::default());
    app.insert_resource(Region::<14>::default());
    app.insert_resource(Region::<15>::default());
    app.insert_resource(Region::<16>::default());
    app.insert_resource(Region::<17>::default());
    app.insert_resource(Region::<18>::default());
    app.insert_resource(Region::<19>::default());
    app.insert_resource(Region::<20>::default());
    app.insert_resource(Region::<21>::default());
    app.insert_resource(Region::<22>::default());
    app.insert_resource(Region::<23>::default());
    app.insert_resource(Region::<24>::default());
    app.insert_resource(Region::<25>::default());
    app.insert_resource(Region::<26>::default());
    app.insert_resource(Region::<27>::default());
    app.insert_resource(Region::<28>::default());
    app.insert_resource(Region::<29>::default());
    app.insert_resource(Region::<30>::default());
    app.insert_resource(Region::<31>::default());
}

pub fn initialize(
    arguments: Res<BoidsArguments>,
    mut window: ResMut<Window>,
    mut debug_draw: ResMut<DebugDraw>,
    mut renderer_state: ResMut<RendererState>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut resources: ResMut<BoidsResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    mut build: nightshade_ui::commands::builder::UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    let starting = arguments.count;
    window.title = "Boids".to_string();

    configure_render(&mut debug_draw, &mut renderer_state, &mut settings);

    let flock_material = insert_material(
        &mut registry,
        FLOCK_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        },
    );

    let boids = population::spawn_boids(storage, starting, 0);
    let draw = spawn_draw(storage, flock_material);
    let lines = spawn_lines(storage);
    let camera = spawn_camera(storage, starting);
    active_camera.0 = Some(camera);
    spawn_flock_sun(&mut commands);

    let hud = {
        let mut tree = UiTreeBuilder::new(build.context());
        let hud = hud::build(&mut tree);
        tree.finish();
        hud
    };

    let schedule = flock::schedule();
    let batches = schedule.batches();
    let regions = regions_for(threads());
    tracing::info!(
        target: "boids",
        "{starting} boids, {regions} of {REGION_COUNT} regions on {} threads, flock batches {batches:?}",
        threads()
    );

    resources.show_boids = true;
    resources.requested_count = starting;
    resources.boids = boids;
    resources.flock = std::sync::Mutex::new(schedule);
    resources.batches = batches;
    resources.draw = Some(draw);
    resources.lines = Some(lines);
    resources.hud = hud;
}

fn spawn_flock_sun(commands: &mut Commands) {
    let sun = spawn_sun_without_shadows(commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.intensity = 1.4;
        }
    });
}

fn configure_render(
    debug_draw: &mut DebugDraw,
    renderer_state: &mut RendererState,
    settings: &mut RenderSettings,
) {
    debug_draw.show_grid = false;
    renderer_state.day_night.auto_cycle = false;
    renderer_state.day_night.hour = 10.5;

    settings.bloom_enabled = false;
    settings.bloom_intensity = 0.0;
    settings.anti_aliasing = AntiAliasing::Off;
    settings.ssao_enabled = false;
    settings.ssr_enabled = false;
    settings.fog = None;
    settings.show_sky = false;
    settings.atmosphere = Atmosphere::None;
    settings.clear_color = [0.02, 0.02, 0.04, 1.0];
}

fn spawn_draw(
    storage: &mut nightshade::ecs::Storage,
    material: nightshade_render::data::asset_id::MaterialId,
) -> Entity {
    let entity = storage.spawn_entities::<(
        Name,
        LocalTransform,
        GlobalTransform,
        Visibility,
        MaterialRef,
        InstancedMesh,
    )>(1)[0];
    storage.set(entity, Name("Flock".to_string()));
    storage.set(entity, LocalTransform::default());
    storage.set(entity, Visibility { visible: true });
    storage.set(entity, MaterialRef::new(material));
    storage.set(entity, InstancedMesh::new("Cone"));
    entity
}

fn spawn_lines(storage: &mut nightshade::ecs::Storage) -> Entity {
    let entity = storage.spawn_entities::<(Lines, Visibility, GlobalTransform)>(1)[0];
    storage.set(entity, Lines::default());
    storage.set(entity, Visibility { visible: true });
    entity
}

fn spawn_camera(storage: &mut nightshade::ecs::Storage, starting: usize) -> Entity {
    let camera = storage.spawn_entities::<(Camera, LocalTransform, GlobalTransform, Name)>(1)[0];
    storage.set(camera, Name("Main Camera".to_string()));
    storage.set(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_near: 0.5,
                z_far: Some(4000.0),
            }),
            smoothing: Some(Smoothing::default()),
        },
    );
    storage.set(
        camera,
        PanOrbitCamera {
            focus: Vec3::zeros(),
            target_focus: Vec3::zeros(),
            radius: bounds_for(starting) * 3.0,
            target_radius: bounds_for(starting) * 3.0,
            pitch: 0.25,
            target_pitch: 0.25,
            yaw: 0.6,
            target_yaw: 0.6,
            ..Default::default()
        },
    );
    camera
}
