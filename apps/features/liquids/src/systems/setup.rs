use crate::data::{Block, atlas_pixels, block_table};
use crate::resources::{VoxelSettings, VoxelWorld, Watcher};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_fly_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::data::config::{DayNightState, RendererState};
use nightshade_render::data::material::{AlphaMode, Material, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use std::sync::Arc;

pub fn initialize(
    mut render_settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut active_camera: ResMut<ActiveCamera>,
    mut registry: ResMut<MaterialRegistry>,
    mut settings: ResMut<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut watcher: ResMut<Watcher>,
    mut textures: TextureLoad,
    mut commands: Commands,
) {
    render_settings.atmosphere = Atmosphere::DayNight;
    render_settings.named_shadow_casters = true;

    let width = theme::ATLAS_COLUMNS * theme::TILE_TEXELS;
    let height = theme::ATLAS_ROWS * theme::TILE_TEXELS;
    queue_decoded_texture(
        &mut textures,
        theme::ATLAS_TEXTURE.to_string(),
        atlas_pixels(),
        width,
        height,
        TextureUsage::Color,
        SamplerSettings {
            wrap_u: SamplerWrap::ClampToEdge,
            wrap_v: SamplerWrap::ClampToEdge,
            mag_filter: SamplerFilter::Nearest,
            min_filter: SamplerFilter::Nearest,
            mipmap_filter: SamplerFilter::Nearest,
        },
    );
    texture_cache_protect(textures.cache, theme::ATLAS_TEXTURE.to_string());

    insert_material(
        &mut registry,
        theme::SOLID_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            atlas_cells: theme::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        &mut registry,
        theme::WATER_MATERIAL,
        Material {
            base_color: theme::WATER_COLOR,
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            base_texture_transform: TextureTransform {
                drift: theme::WATER_DRIFT,
                ..Default::default()
            },
            atlas_cells: theme::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            roughness: 0.3,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        &mut registry,
        theme::LAVA_MATERIAL,
        Material {
            base_color: theme::LAVA_COLOR,
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            base_texture_transform: TextureTransform {
                drift: theme::LAVA_DRIFT,
                ..Default::default()
            },
            atlas_cells: theme::ATLAS_CELLS,
            emissive_factor: theme::LAVA_GLOW,
            sky_light: true,
            vertex_occlusion: true,
            roughness: 0.8,
            metallic: 0.0,
            ..Default::default()
        },
    );

    world.blocks = Arc::new(block_table());
    *settings = VoxelSettings {
        solid_material: theme::SOLID_MATERIAL.to_string(),
        cutout_material: theme::SOLID_MATERIAL.to_string(),
        floor_block: Block::Stone.index(),
        ..Default::default()
    };

    let sun = spawn_sun(&mut commands);
    renderer_state.day_night = DayNightState {
        hour: theme::START_HOUR,
        speed: 0.0,
        auto_cycle: false,
        sun_entity: Some(sun),
        ..Default::default()
    };

    let camera = spawn_fly_camera_at(
        &mut commands,
        theme::EYE,
        theme::YAW,
        theme::PITCH,
        "Camera".to_string(),
    );
    active_camera.0 = Some(camera);
    watcher.camera = Some(camera);
}
