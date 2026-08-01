use crate::data::{BASE_VIEW_DEGREES, EYE_OFFSET, GAMEPAD_LOOK_RATE, GRAVITY, START};
use crate::resources::{Screen, TetherResources};
use crate::systems::{hud, screens, textures, world};
use crate::theme::{
    BLOOM_INTENSITY, BLOOM_THRESHOLD, CONTRAST, EXPOSURE_STOPS, FOG_COLOR, FOG_DENSITY, FOG_END,
    FOG_START, SATURATION, SUN_INTENSITY, SUN_SHADOW_DISTANCE, SUN_TINT, VIGNETTE_INTENSITY,
    VIGNETTE_RADIUS,
};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_render::components::Light;
use nightshade_render::data::config::{Fog, FogMode};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_ui_menu::prelude::{MenuStyle, Menus};

pub fn shape_the_air(mut settings: ResMut<RenderSettings>, mut physics: ResMut<PhysicsWorld>) {
    physics.gravity = Vec3::new(0.0, GRAVITY, 0.0);
    physics.enabled = false;

    settings.atmosphere = Atmosphere::Sunset;
    settings.fog = Some(Fog {
        color: FOG_COLOR,
        start: FOG_START,
        end: FOG_END,
        mode: FogMode::Exponential,
        density: FOG_DENSITY,
        ..Default::default()
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = BLOOM_INTENSITY;
    settings.bloom_threshold = BLOOM_THRESHOLD;
    settings.color_grading.exposure_compensation_ev = EXPOSURE_STOPS;
    settings.color_grading.vignette_intensity = VIGNETTE_INTENSITY;
    settings.color_grading.vignette_radius = VIGNETTE_RADIUS;
    settings.color_grading.saturation = SATURATION;
    settings.color_grading.contrast = CONTRAST;
}

pub fn build_the_canyon(
    mut resources: ResMut<TetherResources>,
    mut registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
    mut load: nightshade_graphics::resources::loading::TextureLoad,
) {
    textures::load_surfaces(&mut load);

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.color = Vec3::new(SUN_TINT[0], SUN_TINT[1], SUN_TINT[2]);
            light.intensity = SUN_INTENSITY;
            light.shadow_distance = SUN_SHADOW_DISTANCE;
        }
    });

    world::build_canyon(&mut commands);
    world::spawn_beacons(&mut registry, &mut commands);

    let runner = world::spawn_runner(&mut commands);
    let rope = world::spawn_rope(&mut registry, &mut commands);
    let camera = spawn_camera(
        &mut commands,
        Vec3::new(START[0], START[1] + EYE_OFFSET, START[2]),
        "Runner Camera".to_string(),
    );
    commands.queue(move |storage, resources| {
        if let Some(entity_camera) = storage.get_mut::<Camera>(camera) {
            if let Projection::Perspective(ref mut perspective) = entity_camera.projection {
                perspective.y_fov_rad = BASE_VIEW_DEGREES.to_radians();
                perspective.z_near = 0.08;
            }
            if let Some(smoothing) = entity_camera.smoothing.as_mut() {
                smoothing.gamepad_sensitivity = GAMEPAD_LOOK_RATE;
            }
        }
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });

    resources.scene.runner = Some(runner);
    resources.scene.rope = Some(rope);
    resources.scene.camera = Some(camera);
}

pub fn build_the_interface(
    mut resources: ResMut<TetherResources>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.ui = hud::build(&mut tree);
    resources.screens = screens::build(&mut tree, &style, &mut menus);
    tree.finish();
}
