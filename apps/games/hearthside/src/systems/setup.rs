use crate::data;
use crate::resources::{Arguments, Lot, Models};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_with_config;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_camera_controller::contract::pan_orbit::{PanOrbitBindings, PanOrbitButton};
use nightshade_gltf::Loading;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::{spawn_mesh_at, spawn_sun};
use nightshade_graphics::prelude::*;
use nightshade_load::{Source, load};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::components::light::Light;
use nightshade_render::data::material::{AlphaMode, Material};
use std::collections::HashMap;

pub fn dress_lot(
    mut settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    mut commands: Commands,
) {
    debug_draw.show_grid = false;
    settings.atmosphere = Atmosphere::Sky;
    settings.color_grading.exposure_compensation_ev = data::EXPOSURE_EV;
    settings.ssao_enabled = true;
    settings.ssao_intensity = data::SSAO_STRENGTH;

    let sun = spawn_sun(&mut commands);
    if let Some(light) = commands.staged_mut::<Light>(sun) {
        light.intensity = data::SUN_STRENGTH;
        light.color = Vec3::new(data::SUN_COLOR[0], data::SUN_COLOR[1], data::SUN_COLOR[2]);
    }
    commands.restage::<Light>(sun);
}

pub fn request_models(mut models: ResMut<Models>, mut loading: ResMut<Loading>) {
    models.prefabs = vec![None; data::CATALOG.len()];
    models.handles = data::CATALOG
        .iter()
        .map(|piece| load(&mut loading, piece_source(piece)))
        .collect();
    models.dweller_handle = Some(load(
        &mut loading,
        Source::Parts {
            head: data::CHARACTER_MODEL.to_vec(),
            files: colormap(data::CHARACTER_COLORMAP),
        },
    ));
}

fn piece_source(piece: &data::Piece) -> Source {
    match piece.kit {
        data::Kit::Furniture => Source::Bytes(piece.bytes.to_vec()),
        data::Kit::Building => Source::Parts {
            head: piece.bytes.to_vec(),
            files: colormap(data::BUILDING_COLORMAP),
        },
    }
}

fn colormap(bytes: &'static [u8]) -> HashMap<String, Vec<u8>> {
    HashMap::from([(data::COLORMAP_URI.to_string(), bytes.to_vec())])
}

pub fn build_lot(
    arguments: Res<Arguments>,
    mut lot: ResMut<Lot>,
    mut registry: ResMut<MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    if let Some(money) = arguments.money {
        lot.money = money as f32;
    }

    register_materials(&mut registry);
    spawn_ground(&mut commands);
    spawn_lot_pad(&mut commands);
    lot.pad = spawn_ghost_pad(&mut commands);
    active_camera.0 = Some(spawn_lot_camera(&mut commands));
    lot.selected = crate::queries::slots(&lot).first().copied().unwrap_or(0);
}

fn register_materials(registry: &mut MaterialRegistry) {
    insert_material(
        registry,
        data::GROUND_MATERIAL,
        Material {
            base_color: data::GROUND_COLOR,
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        data::LOT_MATERIAL,
        Material {
            base_color: data::LOT_COLOR,
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        data::GHOST_CLEAR_MATERIAL,
        translucent(data::GHOST_CLEAR_COLOR),
    );
    insert_material(
        registry,
        data::GHOST_BLOCKED_MATERIAL,
        translucent(data::GHOST_BLOCKED_COLOR),
    );
}

fn translucent(color: [f32; 4]) -> Material {
    Material {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        roughness: 1.0,
        metallic: 0.0,
        unlit: true,
        double_sided: true,
        ..Default::default()
    }
}

fn spawn_ground(commands: &mut Commands) {
    let entity = spawn_mesh_at(
        commands,
        "Plane",
        Vec3::new(0.0, data::GRASS_DROP, 0.0),
        Vec3::new(70.0, 1.0, 70.0),
    );
    set_mesh_material(commands, entity, data::GROUND_MATERIAL);
}

fn spawn_lot_pad(commands: &mut Commands) {
    let entity = spawn_mesh_at(
        commands,
        "Plane",
        Vec3::new(0.0, data::GRASS_DROP + 0.01, 0.0),
        Vec3::new(data::LOT_HALF_WIDTH, 1.0, data::LOT_HALF_DEPTH),
    );
    set_mesh_material(commands, entity, data::LOT_MATERIAL);
}

fn spawn_ghost_pad(commands: &mut Commands) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        "Plane",
        data::PARKED,
        Vec3::new(data::GRID * 0.5, 1.0, data::GRID * 0.5),
    );
    set_mesh_material(commands, entity, data::GHOST_CLEAR_MATERIAL);
    entity
}

fn spawn_lot_camera(commands: &mut Commands) -> Entity {
    let focus = Vec3::new(
        data::CAMERA_FOCUS[0],
        data::CAMERA_FOCUS[1],
        data::CAMERA_FOCUS[2],
    );
    let mut config = PanOrbitCamera::new(focus, data::CAMERA_RADIUS)
        .with_yaw_pitch(data::CAMERA_YAW, data::CAMERA_PITCH);
    config.bindings = PanOrbitBindings {
        orbit: PanOrbitButton::Middle,
        pan: PanOrbitButton::Right,
        orbit_modifier: None,
        pan_modifier: None,
    };
    config.limits.zoom_lower = data::CAMERA_ZOOM_NEAR;
    config.limits.zoom_upper = Some(data::CAMERA_ZOOM_FAR);
    spawn_pan_orbit_camera_with_config(commands, "Lot Camera".to_string(), config)
}
