use crate::resources::DebugViewsResources;
use crate::systems::panel;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_gltf::queries::sample_model;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::queries::lights::sun_rotation;

pub fn initialize(
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut debug_views_resources: ResMut<DebugViewsResources>,
    mut command_queues: ResMut<CommandQueues>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut command_queues, &mut render_settings);
    let handle = nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Path(sample_model(theme::MODEL)),
    );
    debug_views_resources.model = Some(handle);
}

pub fn stand_the_scene(
    mut debug_views_resources: ResMut<DebugViewsResources>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut window: ResMut<nightshade_platform::prelude::Window>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = debug_views_resources.model else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut loading, handle) else {
        return;
    };
    debug_views_resources.model = None;
    let framed = bring_in_the_model(storage, parts, &mut window, outcome);
    let camera = {
        let commands = &mut commands;
        {
            let camera = spawn_pan_orbit_camera_at(
                commands,
                framed.middle,
                framed.reach * theme::CAMERA_BACK,
                theme::CAMERA_YAW,
                theme::CAMERA_PITCH,
                "Camera".to_string(),
            );
            commands.spawn((
                Name("Sun".to_string()),
                LocalTransform {
                    translation: Vec3::new(0.0, framed.reach * 4.0, 0.0),
                    rotation: sun_rotation(theme::SUN_YAW, theme::SUN_PITCH),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                GlobalTransform::default(),
                Light {
                    light_type: LightType::Directional,
                    color: Vec3::new(1.0, 0.96, 0.90),
                    intensity: 4.0,
                    cast_shadows: true,
                    shadow_bias: 0.02,
                    ..Default::default()
                },
            ));

            let ground = spawn_mesh_at(
                commands,
                "Cube",
                Vec3::new(0.0, framed.middle.y - framed.reach * 1.02, 0.0),
                Vec3::new(
                    framed.reach * theme::GROUND,
                    framed.reach * 0.02,
                    framed.reach * theme::GROUND,
                ),
            );
            queue_set_material(
                commands,
                ground,
                "pale floor",
                nightshade_render::data::material::Material {
                    base_color: [0.82, 0.80, 0.76, 1.0],
                    roughness: 0.85,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
            commands.set(ground, nightshade_render::components::mesh::DebugViewExempt);

            camera
        }
    };
    active_camera.0 = Some(camera);
}

pub fn build_panel(
    mut resources: ResMut<DebugViewsResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    resources.handles = panel::build(build.context());
}

fn settle_the_look(command_queues: &mut CommandQueues, settings: &mut RenderSettings) {
    load_hdr_skybox(&mut *command_queues, theme::SKY.to_vec());
    settings.atmosphere = Atmosphere::Hdr;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.3;
    settings.anti_aliasing = AntiAliasing::Taa;
}

struct Framed {
    middle: Vec3,
    reach: f32,
}

fn bring_in_the_model(
    storage: &mut nightshade::ecs::Storage,
    parts: nightshade_prefab::PrefabParts<'_>,
    window: &mut nightshade_platform::prelude::Window,
    outcome: Result<nightshade_prefab::data::ModelImport, String>,
) -> Framed {
    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("{} did not load: {error}", theme::MODEL);
            close_for_want_of_a_model(window);
            return Framed {
                middle: Vec3::zeros(),
                reach: 1.0,
            };
        }
    };
    let Some(prefab) = import.prefabs.first().cloned() else {
        close_for_want_of_a_model(window);
        return Framed {
            middle: Vec3::zeros(),
            reach: 1.0,
        };
    };
    let (middle, reach) = nightshade_prefab::queries::prefab_reach(&prefab, &import.meshes);
    let mut spawn = nightshade_prefab::PrefabSpawn { storage, parts };
    nightshade_prefab::queue_model_load(&mut spawn, &mut import);
    nightshade_prefab::spawn_prefab(&mut spawn, &prefab, Vec3::zeros());
    Framed { middle, reach }
}

fn close_for_want_of_a_model(window: &mut nightshade_platform::prelude::Window) {
    window.should_exit = true;
}
