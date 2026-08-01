use crate::commands::dress;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::material::Material;

pub fn initialize(
    mut commands: Commands,
    mut active_camera: ResMut<ActiveCamera>,
    mut mesh_cache: ResMut<nightshade_render::resources::mesh_cache::MeshCache>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut render_settings);
    bring_in_the_mesh(&mut mesh_cache);
    let camera = {
        let commands = &mut commands;
        let camera = spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_DISTANCE,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        );
        commands.spawn((
            Name("Sun".to_string()),
            LocalTransform {
                translation: Vec3::new(0.0, 400.0, 0.0),
                rotation: nalgebra_glm::quat_angle_axis(theme::SUN_YAW, &Vec3::y())
                    * nalgebra_glm::quat_angle_axis(theme::SUN_PITCH, &Vec3::x()),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Directional,
                color: Vec3::new(1.0, 0.96, 0.90),
                intensity: 3.0,
                cast_shadows: false,
                ..Default::default()
            },
        ));

        let lattice = spawn_lattice(commands);
        dress(
            commands,
            lattice,
            Material {
                base_color: [0.72, 0.60, 0.42, 1.0],
                roughness: 0.55,
                metallic: 0.1,
                ..Default::default()
            },
        );

        camera
    };
    active_camera.0 = Some(camera);
}

fn spawn_lattice(commands: &mut Commands<'_>) -> Entity {
    let half = (theme::SIDE as f32 - 1.0) * 0.5;
    let mut copies = Vec::with_capacity(theme::COPIES);
    for row in 0..theme::SIDE {
        for column in 0..theme::SIDE {
            for depth in 0..theme::SIDE {
                let at = Vec3::new(
                    (column as f32 - half) * theme::PITCH,
                    (row as f32 - half) * theme::PITCH,
                    (depth as f32 - half) * theme::PITCH,
                );
                copies.push(InstanceTransform::from_translation_scale(
                    at,
                    Vec3::new(theme::SCALE, theme::SCALE, theme::SCALE),
                ));
            }
        }
    }
    commands.spawn((
        Name("Lattice".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Visibility { visible: true },
        InstancedMesh::with_instances(theme::MESH, copies),
    ))
}

fn bring_in_the_mesh(cache: &mut nightshade_render::resources::mesh_cache::MeshCache) {
    if !cache.registry.name_to_index.contains_key(theme::MESH) {
        mesh_cache_insert(
            cache,
            theme::MESH.to_string(),
            nightshade_render::data::procedural_meshes::create_cube_mesh(),
        );
    }
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.bloom_enabled = false;
    settings.anti_aliasing = AntiAliasing::Taa;
}
