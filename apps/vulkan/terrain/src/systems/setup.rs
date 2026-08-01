use crate::commands::hold_the_materials;
use crate::queries::{boulder_mesh, boulder_mesh_name};
use crate::resources::{Arguments, Noises, Plan, Terrain};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::components::{Camera, PerspectiveCamera, Projection, Smoothing};
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_fly_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::data::config::Fog;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_cache::MeshCache;

pub fn initialize(
    arguments: Res<Arguments>,
    mut plan: ResMut<Plan>,
    mut terrain: ResMut<Terrain>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut meshes: ResMut<MeshCache>,
    mut active: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    if let Some(radius) = arguments.radius {
        plan.radius = radius.max(1);
    }
    if let Some(cells) = arguments.cells {
        plan.cells = cells.clamp(16, 256);
    }
    if let Some(boulders) = arguments.boulders {
        plan.boulders = boulders;
    }
    if let Some(seed) = arguments.seed {
        plan.seed = seed;
        terrain.noises = Noises::new(seed);
    }

    settings.atmosphere = Atmosphere::CloudySky;
    settings.bloom_enabled = true;
    settings.bloom_intensity = theme::BLOOM_INTENSITY;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.color_grading.exposure = theme::EXPOSURE;

    let reach = plan.radius as f32 * theme::TILE_SPAN;
    settings.fog = Some(Fog {
        color: theme::FOG_COLOR,
        start: reach * theme::FOG_START_SHARE,
        end: reach * theme::FOG_END_SHARE,
        ..Default::default()
    });

    hold_the_materials(&mut registry);

    for kind in 0..theme::BOULDER_KINDS {
        let name = boulder_mesh_name(kind);
        let mesh = boulder_mesh(
            plan.seed.wrapping_add(kind as u32 * 977),
            theme::BOULDER_SEGMENTS,
        );
        mesh_cache_insert(&mut meshes, name.clone(), mesh);
        mesh_cache_protect(&mut meshes, name);
    }

    spawn_sun(&mut commands);

    let camera = spawn_fly_camera_at(
        &mut commands,
        theme::CAMERA_START,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Camera".to_string(),
    );
    commands.set(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: theme::CAMERA_FIELD_OF_VIEW.to_radians(),
                z_far: None,
                z_near: theme::CAMERA_NEAR,
            }),
            smoothing: Some(Smoothing {
                move_speed: theme::CAMERA_SPEED,
                sprint_multiplier: theme::CAMERA_SPRINT,
                ..Default::default()
            }),
        },
    );
    active.0 = Some(camera);
}
