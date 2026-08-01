use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_gltf::queries::sample_model;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::config::Fog;
use nightshade_render::data::material::Material;
use nightshade_render::queries::lights::sun_rotation;

pub fn initialize(
    mut commands: Commands,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut pending: ResMut<crate::resources::Pending>,
    mut active_camera: ResMut<ActiveCamera>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut render_settings);
    let handle = nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Path(sample_model(theme::MODEL)),
    );
    pending.model = Some(handle);
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
                translation: Vec3::new(0.0, 20.0, 0.0),
                rotation: sun_rotation(theme::SUN_YAW, theme::SUN_PITCH),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Directional,
                color: theme::SUN_COLOR,
                intensity: theme::SUN_INTENSITY,
                cast_shadows: true,
                shadow_bias: 0.02,
                ..Default::default()
            },
        ));

        commands.spawn((
            Name("Lantern".to_string()),
            LocalTransform {
                translation: theme::LANTERN,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Point,
                color: theme::LANTERN_COLOR,
                intensity: theme::LANTERN_INTENSITY,
                range: theme::LANTERN_RANGE,
                cast_shadows: true,
                ..Default::default()
            },
        ));
        let flame = spawn_mesh_at(
            commands,
            "Sphere",
            theme::LANTERN,
            Vec3::new(0.12, 0.12, 0.12),
        );
        queue_set_material(
            commands,
            flame,
            "flame",
            Material {
                base_color: [0.0, 0.0, 0.0, 1.0],
                emissive_factor: [
                    theme::LANTERN_COLOR.x * 5.0,
                    theme::LANTERN_COLOR.y * 5.0,
                    theme::LANTERN_COLOR.z * 5.0,
                ],
                unlit: true,
                ..Default::default()
            },
        );

        let ground = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(0.0, -0.05, 0.0),
            Vec3::new(theme::GROUND, 0.05, theme::GROUND),
        );
        queue_set_material(
            commands,
            ground,
            "pale floor",
            Material {
                base_color: [0.80, 0.78, 0.74, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );

        let first = -(theme::PILLARS - 1) as f32 * 0.5;
        for index in 0..theme::PILLARS {
            let across = (first + index as f32) * theme::PILLAR_SPACING;
            let pillar = spawn_mesh_at(
                commands,
                "Cube",
                Vec3::new(across, theme::PILLAR_HEIGHT * 0.5, theme::PILLAR_BEYOND),
                Vec3::new(
                    theme::PILLAR_THICKNESS,
                    theme::PILLAR_HEIGHT,
                    theme::PILLAR_THICKNESS,
                ),
            );
            queue_set_material(
                commands,
                pillar,
                format!("pillar {index}"),
                Material {
                    base_color: [0.62, 0.60, 0.57, 1.0],
                    roughness: 0.85,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
        }

        camera
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.fog = Some(Fog {
        color: theme::FOG_COLOR,
        start: 0.0,
        end: theme::FOG_END,
        volumetric: true,
        density: theme::FOG_DENSITY,
        anisotropy: theme::FOG_ANISOTROPY,
        height: theme::FOG_HEIGHT,
        height_falloff: theme::FOG_HEIGHT_FALLOFF,
        ..Fog::default()
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.25;
    settings.anti_aliasing = AntiAliasing::Taa;
}

pub fn stand_the_model(
    mut pending: ResMut<crate::resources::Pending>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut window: ResMut<nightshade_platform::prelude::Window>,
    parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = pending.model else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut loading, handle) else {
        return;
    };
    pending.model = None;
    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("{} did not load: {error}", theme::MODEL);
            close_for_want_of_a_model(&mut window);
            return;
        }
    };
    let Some(prefab) = import.prefabs.first().cloned() else {
        close_for_want_of_a_model(&mut window);
        return;
    };
    let (middle, reach) = nightshade_prefab::queries::prefab_reach(&prefab, &import.meshes);
    let lowest = nightshade_prefab::queries::prefab_bounds(&prefab, &import.meshes)
        .map(|bounds| bounds.0.y)
        .unwrap_or(middle.y);
    let root = {
        let mut spawn = nightshade_prefab::PrefabSpawn { storage, parts };
        nightshade_prefab::queue_model_load(&mut spawn, &mut import);
        nightshade_prefab::spawn_prefab(&mut spawn, &prefab, Vec3::zeros())
    };
    if reach > 0.0
        && let Some(transform) = storage.get_mut::<LocalTransform>(root)
    {
        let factor = theme::MODEL_REACH / reach;
        let turn = nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::new(0.0, 1.0, 0.0));
        transform.scale = Vec3::new(factor, factor, factor);
        transform.rotation = turn;
        transform.translation = nalgebra_glm::quat_rotate_vec3(
            &turn,
            &(-Vec3::new(middle.x, lowest, middle.z) * factor),
        );
    }
}

fn close_for_want_of_a_model(window: &mut nightshade_platform::prelude::Window) {
    window.should_exit = true;
}
