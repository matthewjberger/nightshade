use crate::commands::{buildings, daylight, life, textures, world};
use crate::data;
use crate::queries::camera::{eye, look};
use crate::queries::town::{lift_at, nearest_walkable};
use crate::resources::{Arguments, Rig, Sheet, Sky, Town, Village};
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::components::light::{Light, LightType};
use nightshade_sprite::prelude::*;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut town: ResMut<Town>,
    mut village: ResMut<Village>,
    mut rig: ResMut<Rig>,
    mut sky: ResMut<Sky>,
    mut sheet: ResMut<Sheet>,
    arguments: Res<Arguments>,
    mut textures_load: TextureLoad,
    mut commands: Commands,
) {
    settings.depth_of_field.enabled = true;
    settings.depth_of_field.focus_distance =
        (data::CAMERA_HEIGHT.powi(2) + data::CAMERA_REACH.powi(2)).sqrt();
    settings.depth_of_field.focus_range = 8.0;
    settings.depth_of_field.max_blur_radius = 4.0;

    let sun = spawn_sun(&mut commands);
    commands.set(
        sun,
        Light {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 0.96, 0.86),
            intensity: data::SUN_STRENGTH,
            range: 100.0,
            cast_shadows: true,
            shadow_bias: 0.0005,
            shadow_softness: data::SUN_SHADOW_SOFTNESS,
            ..Default::default()
        },
    );
    sky.hour = arguments.hour.unwrap_or(data::HOUR);
    daylight::start_the_day(&mut settings);
    village.sun = Some(sun);

    textures::load_block_textures(&mut textures_load);
    textures::insert_materials(&mut registry);
    *sheet = textures::pack_the_sheet(&mut atlases);

    let turntable = commands.spawn((
        Name("Turning World".to_string()),
        LocalTransform {
            translation: Vec3::new(0.0, -data::RADIUS, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
    ));
    village.turntable = Some(turntable);

    world::build(&mut town);
    buildings::spawn_cottages(&mut commands, &mut town, turntable);
    buildings::spawn_lanterns(&mut commands, &mut town, turntable);
    buildings::spawn_board(&mut commands, &mut town, turntable);
    world::spawn_blocks(&mut commands, &town, turntable);
    world::spawn_water(&mut commands, &town, turntable);
    world::spawn_sea_apron(&mut commands, &town, turntable);
    world::spawn_decks(&mut commands, &town, turntable);

    life::spawn_town_tree(&mut commands, &sheet, &mut town, turntable);
    life::spawn_shop_signs(&mut commands, &sheet, &town, turntable);
    life::spawn_plants(&mut commands, &sheet, &mut town, turntable);
    life::spawn_props(&mut commands, &sheet, &mut town, turntable);
    life::spawn_villagers(&mut commands, &sheet, &town, turntable);
    life::spawn_animals(&mut commands, &sheet, &town, turntable);
    life::spawn_loose_goods(&mut commands, &sheet, &town, turntable);

    let start = nearest_walkable(
        &town,
        Vec2::new(
            arguments.column.unwrap_or(data::START_COLUMN),
            data::START_ROW,
        ),
    );
    let lift = lift_at(&town, start);
    let player = life::spawn_player(&mut commands, &sheet, &town, turntable, start);
    village.player = Some(player);
    village.feet = start;
    village.lift = lift;
    rig.focus = start;
    rig.lift = lift;

    let camera = commands.spawn((
        Name("Village Camera".to_string()),
        LocalTransform {
            translation: eye(start, lift),
            rotation: look(start, lift),
            ..Default::default()
        },
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: data::CAMERA_FOV_DEGREES.to_radians(),
                z_far: Some(data::CAMERA_FAR),
                z_near: data::CAMERA_NEAR,
            }),
            smoothing: None,
        },
    ));
    village.camera = Some(camera);
    commands.queue(move |_, resources| {
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
}
