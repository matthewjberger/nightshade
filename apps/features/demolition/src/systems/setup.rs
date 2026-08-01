use crate::data::{Block, atlas_pixels, block_table};
use crate::resources::{Aim, VoxelSettings, VoxelViewer, VoxelWorld, Watcher};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::components::{Camera, PerspectiveCamera, Projection};
use nightshade_camera::prelude::ActiveCamera;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::commands::primitives::register_primitive_mesh;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::config::{DayNightState, RendererState};
use nightshade_render::data::material::{AlphaMode, Material};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_state::MeshRenderState;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};
use std::sync::Arc;

pub fn initialize(
    mut render_settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut active_camera: ResMut<ActiveCamera>,
    mut registry: ResMut<MaterialRegistry>,
    mut settings: ResMut<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut viewer: ResMut<VoxelViewer>,
    mut watcher: ResMut<Watcher>,
    mut aim: ResMut<Aim>,
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
        theme::CUTOUT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            atlas_cells: theme::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Mask,
            alpha_cutoff: 0.5,
            double_sided: true,
            roughness: 0.4,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        &mut registry,
        theme::AIM_MATERIAL,
        Material {
            base_color: theme::AIM_COLOR,
            emissive_factor: theme::AIM_GLOW,
            alpha_mode: AlphaMode::Blend,
            roughness: 0.6,
            metallic: 0.0,
            ..Default::default()
        },
    );

    world.blocks = Arc::new(block_table());
    *settings = VoxelSettings {
        solid_material: theme::SOLID_MATERIAL.to_string(),
        cutout_material: theme::CUTOUT_MATERIAL.to_string(),
        floor_block: Block::Ground.index(),
        block_size: theme::BLOCK_SIZE,
        ..Default::default()
    };
    viewer.whole = true;

    let sun = spawn_sun(&mut commands);
    renderer_state.day_night = DayNightState {
        hour: theme::START_HOUR,
        speed: 0.0,
        auto_cycle: false,
        sun_entity: Some(sun),
        ..Default::default()
    };

    let transform = LocalTransform {
        translation: theme::EYE,
        ..Default::default()
    };
    let camera = commands.spawn((
        Name("Camera".to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: theme::CAMERA_FOV_DEGREES.to_radians(),
                z_far: None,
                z_near: theme::CAMERA_NEAR,
            }),
            smoothing: None,
        },
    ));
    active_camera.0 = Some(camera);
    watcher.camera = Some(camera);

    let marker = commands.spawn((
        Name("Blast marker".to_string()),
        LocalTransform {
            scale: Vec3::repeat(theme::BLAST_RADIUS as f32 * theme::BLOCK_SIZE),
            ..Default::default()
        },
        GlobalTransform::default(),
        RenderMesh::new(theme::AIM_MESH),
        MaterialRef::default(),
        Visibility { visible: false },
    ));
    commands.queue(move |_, resources| {
        register_primitive_mesh(resources, theme::AIM_MESH);
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), marker);
    });
    set_mesh_material(&mut commands, marker, theme::AIM_MATERIAL);
    aim.marker = Some(marker);
}
