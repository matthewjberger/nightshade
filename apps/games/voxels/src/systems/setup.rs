use crate::commands::decode_atlas;
use crate::commands::{
    insert_materials, load_atlas, load_ripples, pack_the_sheet, register_item_meshes, spawn_crack,
    spawn_outline,
};
use crate::data::{BlockKind, Palette};
use crate::queries::boat_mesh;
use crate::resources::{Aim, Player, Sheet, VoxelSettings};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::components::light::Light;
use nightshade_render::data::config::{DayNightState, RendererState};
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};
use nightshade_sprite::prelude::SpriteAtlases;

pub fn initialize(
    mut render_settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut active_camera: ResMut<ActiveCamera>,
    mut registry: ResMut<MaterialRegistry>,
    mut mesh_cache: ResMut<MeshCache>,
    mut atlases: ResMut<SpriteAtlases>,
    mut sheet: ResMut<Sheet>,
    mut player: ResMut<Player>,
    mut blocks: ResMut<VoxelSettings>,
    mut aim: ResMut<Aim>,
    mut textures: TextureLoad,
    mut commands: Commands,
) {
    render_settings.atmosphere = Atmosphere::DayNight;
    render_settings.named_shadow_casters = true;
    render_settings.bloom_enabled = true;
    render_settings.bloom_intensity = theme::BLOOM_DAY;
    render_settings.bloom_threshold = 1.15;
    render_settings.bloom_knee = 0.5;

    let (pixels, width, height) = decode_atlas(Palette::Classic);
    register_item_meshes(&mut mesh_cache, &pixels, width);
    load_atlas(&mut textures, pixels, width, height);
    load_ripples(&mut textures);
    mesh_cache_insert(&mut mesh_cache, theme::BOAT_MESH.to_string(), boat_mesh());
    mesh_cache_protect(&mut mesh_cache, theme::BOAT_MESH.to_string());
    insert_materials(&mut registry);
    *sheet = pack_the_sheet(&mut atlases);
    *blocks = VoxelSettings {
        solid_material: theme::SOLID_MATERIAL.to_string(),
        cutout_material: theme::CUTOUT_MATERIAL.to_string(),
        transparent_material: theme::TRANSPARENT_MATERIAL.to_string(),
        floor_block: BlockKind::Bedrock.index(),
        ..Default::default()
    };

    let sun = spawn_sun(&mut commands);
    commands.set(
        sun,
        Light {
            shadow_softness: theme::SUN_SHADOW_SOFTNESS,
            ..Light::default().with_shadows(0.0005)
        },
    );
    renderer_state.day_night = DayNightState {
        hour: theme::START_HOUR,
        speed: theme::DAY_SPEED,
        auto_cycle: true,
        sun_entity: Some(sun),
        ..Default::default()
    };

    let camera = commands.spawn((
        Name("Player Camera".to_string()),
        LocalTransform::from_translation(player.position),
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
    active_camera.0 = Some(camera);
    player.camera = Some(camera);

    aim.outline = Some(spawn_outline(&mut commands));
    aim.crack = Some(spawn_crack(&mut commands));
}
