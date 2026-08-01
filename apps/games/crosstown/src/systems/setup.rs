use crate::commands::{fighters, interface, textures};
use crate::data;
use crate::resources::{Arguments, Interface, Mode, Models, Ring, Sheets, Talk};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_gltf::Loading;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_load::{Source, load};
use nightshade_render::components::light::Light;
use nightshade_sprite::prelude::SpriteAtlases;
use std::collections::HashMap;

pub fn dress_scene(
    mut settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    mut commands: Commands,
) {
    debug_draw.show_grid = false;
    settings.atmosphere = Atmosphere::Nebula;
    settings.clear_color = data::CLEAR_COLOR;
    settings.bloom_enabled = true;
    settings.bloom_intensity = data::BLOOM;
    settings.bloom_threshold = 1.1;
    settings.bloom_knee = 0.5;

    let moon = spawn_sun(&mut commands);
    commands.set(
        moon,
        Light {
            color: data::MOON_COLOR,
            intensity: data::MOON_STRENGTH,
            ..Light::default().with_shadows(0.0006)
        },
    );
}

pub fn request_models(mut models: ResMut<Models>, mut loading: ResMut<Loading>) {
    let files: HashMap<String, Vec<u8>> = data::MODEL_TEXTURES
        .iter()
        .map(|(name, bytes)| (name.to_string(), bytes.to_vec()))
        .collect();
    models.prefabs = vec![None; data::MODEL_COUNT];
    models.handles = data::MODEL_BYTES
        .iter()
        .map(|gltf| {
            load(
                &mut loading,
                Source::Parts {
                    head: gltf.to_vec(),
                    files: files.clone(),
                },
            )
        })
        .collect();
}

pub fn build_world(
    arguments: Res<Arguments>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut sheets: ResMut<Sheets>,
    mut ring: ResMut<Ring>,
    mut textures: TextureLoad,
    mut commands: Commands,
) {
    textures::load_textures(&mut textures);
    textures::insert_materials(&mut registry);
    *sheets = textures::pack_sheets(&mut atlases);

    let start = Vec3::new(data::STREET_START + 9.0, 0.0, 0.6);
    ring.hero = fighters::spawn_hero(&mut commands, &sheets, start);
    ring.feet = start;
    ring.hero_health = data::HERO_HEALTH;
    ring.camera_x = start.x;
    ring.mark = 1.0;
    ring.mode = if arguments.arcade {
        Mode::Arcade
    } else {
        Mode::Story
    };
    ring.stage = arguments.stage.unwrap_or(0).min(data::STAGE_COUNT - 1);
    ring.wave = arguments.wave.unwrap_or(0);
    ring.piece = 1;
    ring.built = false;
    ring.opened = false;

    let eye = crate::queries::eye_at(start.x, 0.0);
    let camera = commands.spawn((
        Name("Street Camera".to_string()),
        LocalTransform {
            translation: eye,
            rotation: camera_look_rotation(eye, crate::queries::look_at(start.x), 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: data::CAMERA_FOV.to_radians(),
                z_near: data::CAMERA_NEAR,
                z_far: Some(data::CAMERA_FAR),
            }),
            smoothing: None,
        },
    ));
    ring.camera = camera;
    commands.queue(move |_, resources| {
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
}

pub fn build_interface(
    mut handles: ResMut<Interface>,
    mut talk: ResMut<Talk>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    *handles = interface::build(&mut tree);
    tree.finish();
    talk.time = 0.0;
}
