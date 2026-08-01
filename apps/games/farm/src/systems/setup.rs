use crate::commands::{buildings, daylight, effects, farming, lanterns, life, textures, world};
use crate::queries::camera::eye_offset;
use crate::queries::terrain;
use crate::resources::{Diorama, Doors, Farmland, Quarry, Rig, Sheet, Terrain};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::components::light::Light;
use nightshade_render::data::config::{Fog, FogMode, RendererState};
use nightshade_sprite::prelude::*;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut terrain: ResMut<Terrain>,
    mut farmland: ResMut<Farmland>,
    mut doors: ResMut<Doors>,
    mut quarry: ResMut<Quarry>,
    mut diorama: ResMut<Diorama>,
    mut sheet: ResMut<Sheet>,
    mut textures: TextureLoad,
    mut commands: Commands,
) {
    settings.bloom_enabled = true;
    settings.bloom_intensity = theme::BLOOM_DAY;
    settings.bloom_threshold = 1.3;
    settings.bloom_knee = 0.5;
    settings.fog = Some(Fog {
        color: [0.74, 0.78, 0.88],
        start: theme::FOG_START,
        end: theme::FOG_END,
        mode: FogMode::Linear,
        ..Default::default()
    });
    settings.depth_of_field.enabled = true;
    settings.depth_of_field.focus_distance = eye_offset(0.0).magnitude();
    settings.depth_of_field.focus_range = 6.0;
    settings.depth_of_field.max_blur_radius = 6.0;

    let sun = spawn_sun(&mut commands);
    commands.set(
        sun,
        Light {
            shadow_softness: theme::SUN_SHADOW_SOFTNESS,
            ..Light::default().with_shadows(0.0005)
        },
    );
    let hour = diorama.start_hour;
    daylight::start_the_clock(&mut settings, &mut renderer_state, sun, hour);
    daylight::spawn_moon(&mut commands);
    diorama.sun = Some(sun);
    diorama.last_hour = hour;

    textures::load_block_textures(&mut textures);
    textures::load_dungeon_textures(&mut textures);
    textures::insert_materials(&mut registry);

    world::build(&mut terrain);
    world::spawn_blocks(&mut commands, &terrain);
    world::spawn_bridge_rails(&mut commands, &terrain);
    world::spawn_water(&mut commands, &terrain);
    world::spawn_sea(&mut commands);
    doors.doors = buildings::spawn_houses(&mut commands, &terrain);
    buildings::spawn_pen_fence(&mut commands, &mut terrain);
    buildings::spawn_well(&mut commands, &mut terrain);
    buildings::spawn_anvil(&mut commands, &mut terrain);
    buildings::spawn_wood_pile(&mut commands, &mut terrain, theme::WOOD_PILE);
    buildings::spawn_campfire(&mut commands, &mut terrain);
    lanterns::spawn(&mut commands, &terrain);
    effects::spawn_fireflies(&mut commands, &terrain, &theme::FIREFLY_SPOTS);
    for (column, row) in crate::queries::world::starting_plots(&terrain) {
        farming::till(&mut commands, &terrain, &mut farmland, column, row);
    }

    *sheet = textures::pack_the_sheet(&mut atlases);
    quarry.spots = life::quarry_spots(&mut terrain);
    life::spawn_trees(&mut commands, &mut terrain, &sheet);
    life::spawn_bushes(&mut commands, &mut terrain, &sheet);
    life::spawn_props(&mut commands, &mut terrain, &sheet);
    life::spawn_village_props(&mut commands, &mut terrain, &sheet);
    life::spawn_animals(&mut commands, &terrain, &sheet);
    life::spawn_people(&mut commands, &terrain, &sheet);
    life::spawn_hollow_door(&mut commands, &mut terrain, &sheet);
    let farmer = life::spawn_farmer(&mut commands, &terrain, &sheet);
    diorama.farmer = Some(farmer);
    let start = terrain::on_ground(&terrain, theme::FARMER_START.x, theme::FARMER_START.y);
    diorama.feet = start;

    let eye = start + eye_offset(0.0);
    let camera = commands.spawn((
        Name("Tilted Camera".to_string()),
        LocalTransform {
            translation: eye,
            rotation: camera_look_rotation(eye, start, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: theme::CAMERA_FOV_DEGREES.to_radians(),
                z_far: Some(theme::CAMERA_FAR),
                z_near: theme::CAMERA_NEAR,
            }),
            smoothing: None,
        },
    ));
    diorama.camera = Some(camera);
    commands.queue(move |_, resources| {
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
        resources.res_mut::<Rig>().focus = start;
    });
}
