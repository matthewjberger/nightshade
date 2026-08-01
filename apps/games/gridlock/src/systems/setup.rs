use crate::commands::figures::spawn_hero;
use crate::commands::flashes::spawn_flash_pool;
use crate::commands::models::prepare_models;
use crate::data::look::{
    BLOOM_THRESHOLD, FOG_COLOR, FOG_START, GROUND_COLOR, GROUND_NAME, GROUND_ROUGHNESS,
    NEON_COLORS, NEON_NAMES, PAVEMENT_COLOR, PAVEMENT_NAME, PAVEMENT_ROUGHNESS, RAIN_COLOR,
    RAIN_HEIGHT, RAIN_REACH,
};
use crate::data::motion::{WALK_CAMERA_MAX, WALK_CAMERA_MIN};
use crate::data::scale::{GROUND_LIFT, GROUND_SIZE};
use crate::data::settings::Settings;
use crate::queries::grid::corner_position;
use crate::resources::{Flashes, GridlockResources, Models, Player};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_third_person_camera;
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_config::contract::Config;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::config::{DayNightState, Fog, FogMode, RendererState};
use nightshade_render::data::material::Material;
use nightshade_text::prelude::TextState;

const START_NODE: [i32; 3] = [0, 0, 2];

pub fn initialize(
    config: Res<Config<Settings>>,
    mut settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut registry: ResMut<MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut resources: ResMut<GridlockResources>,
    mut player: ResMut<Player>,
    mut models: ResMut<Models>,
    mut flashes: ResMut<Flashes>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    settings.fog = Some(Fog {
        color: FOG_COLOR,
        start: FOG_START,
        end: config.number(Settings::FogReach),
        mode: FogMode::Linear,
        volumetric: false,
        density: 0.0,
        anisotropy: 0.55,
        height: 0.0,
        height_falloff: 0.03,
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = config.number(Settings::Bloom);
    settings.bloom_threshold = BLOOM_THRESHOLD;
    settings.ssr_enabled = true;
    settings.color_grading.contrast = config.number(Settings::Contrast);
    settings.color_grading.saturation = config.number(Settings::Saturation);
    settings.color_grading.vignette_intensity = config.number(Settings::Vignette);

    let sun = spawn_sun(&mut commands);
    settings.atmosphere = Atmosphere::DayNight;
    renderer_state.day_night = DayNightState {
        hour: config.number(Settings::StartHour),
        speed: config.number(Settings::DaySpeed),
        auto_cycle: true,
        sun_entity: Some(sun),
        ..Default::default()
    };

    let ground = spawn_mesh_at(
        &mut commands,
        "Plane",
        Vec3::new(0.0, GROUND_LIFT, 0.0),
        Vec3::new(GROUND_SIZE, 1.0, GROUND_SIZE),
    );
    let asphalt = insert_material(
        &mut registry,
        GROUND_NAME,
        Material {
            base_color: GROUND_COLOR,
            roughness: GROUND_ROUGHNESS,
            metallic: 0.05,
            ..Default::default()
        },
    );
    commands.set(ground, MaterialRef::new(asphalt));
    resources.pavement_material = Some(insert_material(
        &mut registry,
        PAVEMENT_NAME,
        Material {
            base_color: PAVEMENT_COLOR,
            roughness: PAVEMENT_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    ));

    let glow = config.number(Settings::NeonGlow);
    resources.neon_materials = NEON_NAMES
        .iter()
        .zip(NEON_COLORS)
        .map(|(name, color)| {
            insert_material(
                &mut registry,
                *name,
                Material {
                    base_color: [color[0], color[1], color[2], 1.0],
                    emissive_factor: color,
                    emissive_strength: glow,
                    roughness: 0.4,
                    ..Default::default()
                },
            )
        })
        .collect();

    prepare_models(&mut models);
    flashes.pool = spawn_flash_pool(
        &mut text_state,
        &mut commands,
        config.number(Settings::FlashSize),
    );

    let start = corner_position(START_NODE);
    let body = spawn_hero(&mut commands, start);
    player.body = Some(body);

    let walk_camera = spawn_third_person_camera(
        &mut commands,
        body,
        config.number(Settings::WalkDistance),
        std::f32::consts::PI,
        config.number(Settings::WalkPitch),
        "Walk Camera".to_string(),
    );
    if let Some(third_person) = commands.staged_mut::<ThirdPersonCamera>(walk_camera) {
        third_person.height_offset = config.number(Settings::WalkHead);
        third_person.distance_min = WALK_CAMERA_MIN;
        third_person.distance_max = WALK_CAMERA_MAX;
        third_person.collision_enabled = false;
    }
    commands.restage::<ThirdPersonCamera>(walk_camera);
    player.walk_camera = Some(walk_camera);

    let eye = start
        + Vec3::new(
            0.0,
            config.number(Settings::ChaseHeight),
            -config.number(Settings::ChaseBack),
        );
    let chase_camera = spawn_camera(&mut commands, eye, "Chase Camera".to_string());
    player.chase_camera = Some(chase_camera);
    let fly_camera = spawn_camera(&mut commands, eye, "Fly Camera".to_string());
    player.fly_camera = Some(fly_camera);
    active_camera.0 = Some(walk_camera);

    let rain = commands.spawn(());
    commands.set(
        rain,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: RAIN_REACH,
            },
            position: start + Vec3::new(0.0, RAIN_HEIGHT, 0.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            spawn_rate: config.number(Settings::Rain),
            particle_lifetime_min: 0.9,
            particle_lifetime_max: 1.1,
            initial_velocity_min: 13.0,
            initial_velocity_max: 15.0,
            velocity_spread: 0.03,
            gravity: Vec3::zeros(),
            drag: 0.0,
            size_start: 0.07,
            size_end: 0.07,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, RAIN_COLOR),
                    (0.8, RAIN_COLOR),
                    (
                        1.0,
                        Vec4::new(RAIN_COLOR.x, RAIN_COLOR.y, RAIN_COLOR.z, 0.0),
                    ),
                ],
            },
            emissive_strength: 0.5,
            enabled: true,
            ..Default::default()
        },
    );
    player.rain = Some(rain);
}
