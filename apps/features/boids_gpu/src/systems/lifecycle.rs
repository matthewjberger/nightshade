use crate::queries::bounds_for;
use crate::resources::{BoidsArguments, BoidsGpuResources};
use crate::systems::hud;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::UiTreeBuilder;

pub fn initialize(
    arguments: Res<BoidsArguments>,
    mut window: ResMut<Window>,
    mut debug_draw: ResMut<DebugDraw>,
    mut renderer_state: ResMut<RendererState>,
    mut settings: ResMut<RenderSettings>,
    mut resources: ResMut<BoidsGpuResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let starting = arguments.count;
    window.title = "Boids GPU".to_string();

    configure_render(&mut debug_draw, &mut renderer_state, &mut settings);
    spawn_camera(&mut commands, &mut active_camera, starting);
    spawn_flock_sun(&mut commands);

    resources.count = starting;
    resources.visible = true;

    let mut tree = UiTreeBuilder::new(build.context());
    hud::build(&mut tree, &mut resources);
    tree.finish();

    tracing::info!(
        target: "boids_gpu",
        "{starting} boids, stepped and drawn without leaving the gpu"
    );
}

fn spawn_camera(commands: &mut Commands, active_camera: &mut ActiveCamera, starting: usize) {
    let camera = commands.spawn((
        Name("Main Camera".to_string()),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_near: 0.5,
                z_far: Some(8000.0),
            }),
            smoothing: Some(Smoothing::default()),
        },
        LocalTransform::default(),
        GlobalTransform::default(),
    ));
    commands.set(
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
    active_camera.0 = Some(camera);
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
