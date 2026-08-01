use crate::commands::city::spawn_city;
use crate::commands::creatures::{
    spawn_animals, spawn_floating_pool, spawn_givers, spawn_guards, spawn_merchants, spawn_mobs,
    spawn_people, spawn_pickups, spawn_ring, spawn_stage_camera, spawn_stage_figure,
    spawn_townsfolk,
};
use crate::commands::dungeons::{spawn_crypt, spawn_fort, spawn_harbor, spawn_mine};
use crate::commands::hamlet::spawn_starter_wood;
use crate::commands::models::prepare_models;
use crate::commands::random::random_name;
use crate::commands::scenery::{
    register_palette, spawn_barrow, spawn_farm, spawn_ground_cover, spawn_reeds, spawn_roads,
    spawn_terrain, spawn_trees, spawn_water, spawn_wood,
};
use crate::commands::sky::{spawn_birds, spawn_clouds};
use crate::data::assets::ITEM_ICONS;
use crate::data::items::{ITEM_ATLAS, ITEM_CELL};
use crate::data::{
    CITY_STREETS, DAY_SPEED, FRAME_IMAGES, ICON_ATLAS, ICON_CELL, KEEP_MAX, KEEP_MIN, MAP_ATLAS,
    MAP_ICON_CELL, MAP_IMAGE, PITS, ROAD_WIDTH, START_HOUR, SUN_SHADOW_BIAS,
    SUN_SHADOW_NORMAL_BIAS, WATER_LEVEL, ZONE_SIZE,
};
use crate::queries::{
    in_city, inside_rect, road_distance, segment_distance, terrain_height, terrain_normal,
};
use crate::resources::{Creation, Flashes, Icons, Models, Screen, ThornmereResources};
use crate::systems::hud::HUD_ICONS;
use crate::systems::menu;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_kit::prelude::{ICONS, MAP_ICONS};
use nightshade_render::contract::Light;
use nightshade_render::data::config::{DayNightState, Fog, FogMode, RendererState};
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::{UiTreeBuilder, ui_upload_image};
use nightshade_ui_menu::prelude::{MenuStyle, Menus};

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut registry: ResMut<MaterialRegistry>,
    mut mesh_cache: ResMut<MeshCache>,
    mut active_camera: ResMut<ActiveCamera>,
    mut resources: ResMut<ThornmereResources>,
    mut flashes: ResMut<Flashes>,
    mut text_state: ResMut<TextState>,
    mut models: ResMut<Models>,
    mut creation: ResMut<Creation>,
    mut commands: Commands,
) {
    let resources = &mut *resources;

    settings.fog = Some(Fog {
        color: [0.72, 0.80, 0.90],
        start: 110.0,
        end: 340.0,
        mode: FogMode::Linear,
        volumetric: false,
        density: 0.0,
        anisotropy: 0.55,
        height: 0.0,
        height_falloff: 0.03,
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.22;

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.shadow_bias = SUN_SHADOW_BIAS;
            light.shadow_normal_bias = SUN_SHADOW_NORMAL_BIAS;
        }
    });
    settings.atmosphere = Atmosphere::DayNight;
    renderer_state.day_night = DayNightState {
        hour: START_HOUR,
        speed: DAY_SPEED,
        auto_cycle: false,
        sun_entity: Some(sun),
        ..Default::default()
    };

    prepare_models(&mut models);

    let palette = register_palette(&mut registry);
    resources.palette = palette.clone();
    spawn_terrain(&mut mesh_cache, &mut commands, &palette);
    spawn_water(&mut commands, &palette, &mut resources.random);
    spawn_roads(&mut commands);
    spawn_city(&mut commands, &palette, &mut resources.random);
    spawn_farm(&mut commands, &palette, &mut resources.random);
    spawn_fort(&mut commands, &palette, &mut resources.random);
    spawn_barrow(&mut commands, &mut resources.random);
    spawn_crypt(&mut commands, &palette, &mut resources.random);
    spawn_mine(&mut commands, &palette, &mut resources.random);
    spawn_harbor(&mut commands, &palette, &mut resources.random);
    spawn_wood(&mut commands, &palette, &mut resources.random);
    spawn_starter_wood(&mut commands, &palette, &mut resources.random);
    spawn_trees(&mut commands, &mut resources.random);
    spawn_reeds(&mut commands, &mut resources.random);
    spawn_ground_cover(&mut commands, &palette, resources.random.u64(..));
    spawn_clouds(&mut commands, &palette, &mut resources.random);
    spawn_birds(&mut commands, &palette, &mut resources.random);

    spawn_mobs(
        &mut resources.random,
        &mut text_state,
        &mut commands,
        &palette,
    );
    spawn_people(&mut resources.random, &mut text_state, &mut commands);
    spawn_guards(&mut text_state, &mut commands);
    spawn_townsfolk(&mut text_state, &mut commands);
    spawn_animals(&mut resources.random, &mut commands);
    spawn_givers(&mut text_state, &mut commands);
    spawn_merchants(&mut text_state, &mut commands);
    spawn_pickups(&mut resources.random, &mut commands, &palette);
    flashes.pool = spawn_floating_pool(&mut text_state, &mut commands);
    resources.ring = Some(spawn_ring(&mut commands));

    let figure = spawn_stage_figure(&mut commands, creation.class);
    let camera = spawn_stage_camera(&mut commands, figure);
    active_camera.0 = Some(camera);
    creation.figure = Some(figure);
    creation.camera = Some(camera);
}

fn pack_atlas(
    names: &[&str],
    source: &[(&str, &[u8])],
    cell: u32,
    side: u32,
) -> (Vec<u8>, Vec<(Vec2, Vec2)>) {
    let side = side as usize;
    let cell = cell as usize;
    let per_row = side / cell;
    let mut pixels = vec![0u8; side * side * 4];
    let mut cells = Vec::new();
    for (slot, name) in names.iter().enumerate() {
        let column = slot % per_row;
        let row = slot / per_row;
        let origin_x = column * cell;
        let origin_y = row * cell;
        if let Some((_, bytes)) = source.iter().find(|(icon, _)| icon == name)
            && let Ok(decoded) = image::load_from_memory(bytes)
        {
            let picture = decoded.to_rgba8();
            let fit = (cell as u32 - 8).min(picture.width().max(picture.height()));
            let scaled = image::imageops::resize(
                &picture,
                fit * picture.width() / picture.width().max(picture.height()),
                fit * picture.height() / picture.width().max(picture.height()),
                image::imageops::FilterType::Triangle,
            );
            let offset_x = origin_x + (cell - scaled.width() as usize) / 2;
            let offset_y = origin_y + (cell - scaled.height() as usize) / 2;
            for (x, y, pixel) in scaled.enumerate_pixels() {
                let index = ((offset_y + y as usize) * side + offset_x + x as usize) * 4;
                pixels[index..index + 4].copy_from_slice(&pixel.0);
            }
        }
        cells.push((
            vec2(origin_x as f32 / side as f32, origin_y as f32 / side as f32),
            vec2(
                (origin_x + cell) as f32 / side as f32,
                (origin_y + cell) as f32 / side as f32,
            ),
        ));
    }
    (pixels, cells)
}

fn whiten(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel[0] = 255;
        pixel[1] = 255;
        pixel[2] = 255;
    }
}

fn shade(color: [f32; 3], light: f32) -> [u8; 4] {
    [
        (color[0] * light * 255.0).clamp(0.0, 255.0) as u8,
        (color[1] * light * 255.0).clamp(0.0, 255.0) as u8,
        (color[2] * light * 255.0).clamp(0.0, 255.0) as u8,
        255,
    ]
}

fn map_picture() -> Vec<u8> {
    let side = MAP_IMAGE as usize;
    let half = ZONE_SIZE / 2.0;
    let mut pixels = vec![0u8; side * side * 4];
    let light_direction = Vec3::new(-0.5, 0.75, -0.45).normalize();
    for row in 0..side {
        for column in 0..side {
            let x = -half + column as f32 / (side - 1) as f32 * ZONE_SIZE;
            let z = -half + row as f32 / (side - 1) as f32 * ZONE_SIZE;
            let spot = Vec2::new(x, z);
            let height = terrain_height(x, z);
            let normal = terrain_normal(x, z);
            let light = 0.55 + 0.6 * normal.dot(&light_direction).max(0.0);
            let border = column < 5 || row < 5 || column >= side - 5 || row >= side - 5;
            let color = if border {
                shade([0.78, 0.66, 0.44], 1.0)
            } else if height < WATER_LEVEL {
                let depth = ((WATER_LEVEL - height) / 5.0).clamp(0.0, 1.0);
                shade(
                    [
                        0.30 - 0.12 * depth,
                        0.52 - 0.16 * depth,
                        0.72 - 0.14 * depth,
                    ],
                    1.0,
                )
            } else if PITS.iter().any(|pit| inside_rect(spot, pit.min, pit.max)) {
                shade([0.34, 0.27, 0.20], light)
            } else if in_city(spot, 0.0) {
                let edge = in_city(spot, 0.0) && !in_city(spot, -2.0);
                let street = CITY_STREETS
                    .iter()
                    .flat_map(|street| street.windows(2))
                    .any(|pair| segment_distance(spot, pair[0], pair[1]) < ROAD_WIDTH * 0.6);
                if edge {
                    shade([0.30, 0.30, 0.32], light)
                } else if street {
                    shade([0.74, 0.66, 0.52], light)
                } else if inside_rect(spot, KEEP_MIN, KEEP_MAX) {
                    shade([0.58, 0.56, 0.54], light)
                } else {
                    shade([0.66, 0.62, 0.56], light)
                }
            } else if road_distance(spot) < ROAD_WIDTH * 0.9 {
                shade([0.74, 0.64, 0.46], light)
            } else {
                let rise = ((height + 1.0) / 8.0).clamp(0.0, 1.0);
                shade(
                    [0.34 + 0.24 * rise, 0.56 + 0.08 * rise, 0.26 + 0.16 * rise],
                    light,
                )
            };
            let index = (row * side + column) * 4;
            pixels[index..index + 4].copy_from_slice(&color);
        }
    }
    pixels
}

pub fn build_interface(
    mut resources: ResMut<ThornmereResources>,
    mut icons: ResMut<Icons>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut creation: ResMut<Creation>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let (pixels, cells) = pack_atlas(&HUD_ICONS, &ICONS, ICON_CELL, ICON_ATLAS);
    let upload = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(images, runtime, &pixels, ICON_ATLAS, ICON_ATLAS)
    };
    if let Some(upload) = upload {
        icons.layer = Some(upload.layer);
        icons.cells = cells
            .iter()
            .map(|(low, high)| {
                (
                    vec2(low.x * upload.uv_max.x, low.y * upload.uv_max.y),
                    vec2(high.x * upload.uv_max.x, high.y * upload.uv_max.y),
                )
            })
            .collect();
    }
    let item_names: Vec<&str> = ITEM_ICONS.iter().map(|(name, _)| *name).collect();
    let (mut pixels, cells) = pack_atlas(&item_names, &ITEM_ICONS, ITEM_CELL, ITEM_ATLAS);
    whiten(&mut pixels);
    let upload = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(images, runtime, &pixels, ITEM_ATLAS, ITEM_ATLAS)
    };
    if let Some(upload) = upload {
        icons.item_layer = Some(upload.layer);
        icons.item_cells = cells
            .iter()
            .map(|(low, high)| {
                (
                    vec2(low.x * upload.uv_max.x, low.y * upload.uv_max.y),
                    vec2(high.x * upload.uv_max.x, high.y * upload.uv_max.y),
                )
            })
            .collect();
    }
    let mark_names: Vec<&str> = MAP_ICONS.iter().map(|(name, _)| *name).collect();
    let (pixels, cells) = pack_atlas(&mark_names, &MAP_ICONS, MAP_ICON_CELL, MAP_ATLAS);
    let upload = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(images, runtime, &pixels, MAP_ATLAS, MAP_ATLAS)
    };
    if let Some(upload) = upload {
        icons.mark_layer = Some(upload.layer);
        icons.marks = cells
            .iter()
            .map(|(low, high)| {
                (
                    vec2(low.x * upload.uv_max.x, low.y * upload.uv_max.y),
                    vec2(high.x * upload.uv_max.x, high.y * upload.uv_max.y),
                )
            })
            .collect();
    }
    for frame_image in FRAME_IMAGES {
        let mut layer = None;
        if let Some((_, bytes)) = ICONS.iter().find(|(icon, _)| *icon == frame_image)
            && let Ok(decoded) = image::load_from_memory(bytes)
        {
            let picture = decoded.to_rgba8();
            let (width, height) = picture.dimensions();
            let (images, runtime) = tree.images_and_runtime();
            layer = ui_upload_image(images, runtime, picture.as_raw(), width, height)
                .map(|upload| upload.layer);
        }
        icons.frame_layers.push(layer);
    }
    let pixels = map_picture();
    let upload = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(images, runtime, &pixels, MAP_IMAGE, MAP_IMAGE)
    };
    if let Some(upload) = upload {
        icons.map_layer = Some(upload.layer);
        icons.map_extent = upload.uv_max;
    }
    let name = random_name(&mut resources.random);
    menu::build(&mut tree, &style, &mut menus, &mut creation, &name);
    tree.finish();
}
