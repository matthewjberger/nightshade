use crate::data;
use crate::resources::{Arguments, BoomstickResources};
use crate::systems::{effects, hud, models, screens, textures};
use crate::tags::{Eyes, Player};
use nightshade::prelude::*;
use nightshade_camera::commands::spawn_first_person_camera;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::ParticleTextures;
use nightshade_physics::commands::spawn_character_body;
use nightshade_physics::contract::components::{
    CharacterControllerComponent, PhysicsInterpolation,
};
use nightshade_physics::resources::PhysicsWorld;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::config::{Fog, FogMode};
use nightshade_render::resources::viewport::{Viewport, ViewportExtent};

pub fn initialize(
    arguments: Res<Arguments>,
    mut render_settings: ResMut<RenderSettings>,
    mut viewport: ResMut<Viewport>,
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<BoomstickResources>,
    style: Res<nightshade_ui_menu::data::MenuStyle>,
    mut menus: ResMut<nightshade_ui_menu::resources::Menus<crate::resources::Screen>>,
    mut texture_load: nightshade_graphics::resources::loading::TextureLoad,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    dress_the_frame(&mut render_settings);
    viewport.extent = ViewportExtent::Unobstructed;
    physics.gravity.y = data::GRAVITY;
    textures::load(&mut texture_load);

    spawn_sun(&mut commands);
    spawn_lobby(&mut commands);
    spawn_player(&mut commands, &mut resources);

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    tree.finish();

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.screens = screens::build(&mut tree, &style, &mut menus);
    tree.finish();

    if let Some(level) = arguments.level {
        resources.mission.level = level.min(data::levels::LEVEL_COUNT - 1);
        resources.mission.retry = true;
        resources.mission.wanted = true;
    }
    resources.shown.health = u32::MAX;
    resources.shown.armor = u32::MAX;
    resources.shown.ammo = u32::MAX;
}

fn dress_the_frame(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::Space;
    settings.material_anisotropy_filtering = data::ANISOTROPY;
    settings.fog = Some(Fog {
        color: data::FOG_COLOR,
        start: data::FOG_START,
        end: data::FOG_END,
        mode: FogMode::Linear,
        ..Default::default()
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.06;
    settings.bloom_threshold = 1.4;
    settings.ssao_enabled = true;
    settings.ssao_intensity = 0.9;
    settings.ssao_radius = 0.6;
    settings.color_grading.contrast = 1.08;
    settings.color_grading.saturation = 1.06;
    settings.color_grading.vignette_intensity = 0.28;
    settings.color_grading.vignette_radius = 0.86;
    settings.color_grading.vignette_smoothness = 0.5;
}

pub fn load_particles(mut textures: ResMut<ParticleTextures>) {
    effects::load(&mut textures);
}

pub fn request_models(
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut resources: ResMut<BoomstickResources>,
) {
    models::request(&mut loading, &mut resources);
}

fn spawn_lobby(commands: &mut Commands) {
    crate::commands::spawn_block(
        commands,
        "boomstick_lobby_floor",
        data::LOBBY_AT + Vec3::new(0.0, -data::STANCE_HEIGHT - 0.5, 0.0),
        Vec3::new(20.0, 1.0, 20.0),
        textures::tiled(
            textures::DECK,
            textures::tiles(20.0, 20.0, 0.12),
            [0.82, 0.84, 0.92, 1.0],
        ),
    );
    crate::commands::spawn_block(
        commands,
        "boomstick_lobby_wall",
        data::LOBBY_AT + Vec3::new(0.0, 2.0, -10.0),
        Vec3::new(20.0, 9.0, 1.0),
        textures::tiled(
            textures::BRICK,
            textures::tiles(20.0, 9.0, 0.16),
            [0.62, 0.68, 0.84, 1.0],
        ),
    );
    let lamp = spawn_light_entity(
        commands,
        data::LOBBY_AT + Vec3::new(0.0, 5.0, -5.0),
        "boomstick_lobby_lamp",
    );
    commands.set(
        lamp,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 0.62, 0.34),
            intensity: 80.0,
            range: 24.0,
            cast_shadows: false,
            ..Default::default()
        },
    );
}

fn spawn_player(commands: &mut Commands, resources: &mut BoomstickResources) {
    let body = spawn_character_body(commands, data::LOBBY_AT, "Player");
    commands.add_tag_type::<Player>(body);
    commands.set(
        body,
        PhysicsInterpolation {
            previous_translation: data::LOBBY_AT,
            previous_rotation: nalgebra_glm::quat_identity(),
            current_translation: data::LOBBY_AT,
            current_rotation: nalgebra_glm::quat_identity(),
            enabled: false,
        },
    );
    commands.set(
        body,
        CharacterControllerComponent {
            max_speed: data::RUN_SPEED,
            acceleration: data::ACCELERATION,
            jump_impulse: data::JUMP_IMPULSE,
            crouch_enabled: false,
            ..CharacterControllerComponent::new_capsule(
                data::CAPSULE_HALF_HEIGHT,
                data::CAPSULE_RADIUS,
            )
        },
    );

    let eyes = spawn_first_person_camera(
        commands,
        body,
        Vec3::new(0.0, data::EYE_HEIGHT, 0.0),
        data::FIELD_OF_VIEW_DEGREES,
    );
    commands.add_tag_type::<Eyes>(eyes);

    resources.player.body = Some(body);
    resources.player.eyes = Some(eyes);
}
