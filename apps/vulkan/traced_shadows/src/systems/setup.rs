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
    mut pending: ResMut<crate::resources::Pending>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut render_settings);
    let model = nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Path(sample_model(theme::MODEL)),
    );
    let masked = nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Path(sample_model(theme::MASKED_MODEL)),
    );
    pending.model = Some(model);
    pending.masked = Some(masked);
}

pub fn stand_the_scene(
    mut commands: Commands,
    mut pending: ResMut<crate::resources::Pending>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut active_camera: ResMut<ActiveCamera>,
    mut window: ResMut<nightshade_platform::prelude::Window>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let (Some(model), Some(masked)) = (pending.model, pending.masked) else {
        return;
    };
    if !nightshade_load::ready(&loading, model) || !nightshade_load::ready(&loading, masked) {
        return;
    }
    let Some(first) = nightshade_load::take(&mut loading, model) else {
        return;
    };
    let Some(second) = nightshade_load::take(&mut loading, masked) else {
        return;
    };
    pending.model = None;
    pending.masked = None;
    let framed = bring_in_the_model(
        storage,
        parts.reborrow(),
        &mut window,
        first,
        Vec3::zeros(),
        None,
    );
    bring_in_the_model(
        storage,
        parts.reborrow(),
        &mut window,
        second,
        Vec3::new(
            framed.middle.x + framed.reach * theme::MASKED_ASIDE,
            0.0,
            framed.middle.z,
        ),
        Some(framed.reach * theme::MASKED_REACH),
    );
    let camera = {
        let commands = &mut commands;
        {
            let camera = spawn_pan_orbit_camera_at(
                commands,
                framed.middle + Vec3::new(framed.reach * theme::MASKED_ASIDE * 0.5, 0.0, 0.0),
                framed.reach * theme::CAMERA_BACK * theme::MASKED_ROOM,
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
                    intensity: 6.0,
                    cast_shadows: true,
                    shadow_bias: 0.02,
                    ..Default::default()
                },
            ));

            let ground = spawn_mesh_at(
                commands,
                "Cube",
                Vec3::new(
                    framed.middle.x + framed.reach * theme::MASKED_ASIDE * 0.5,
                    framed.reach * -0.02,
                    framed.middle.z,
                ),
                Vec3::new(
                    framed.reach * theme::GROUND,
                    framed.reach * 0.02,
                    framed.reach * theme::GROUND,
                ),
            );
            let name = "pale floor".to_string();
            commands.queue({
                let name = name.clone();
                move |storage, resources| {
                    let id = material_registry_insert(
                        resources
                            .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                        name.clone(),
                        nightshade_render::data::material::Material {
                            base_color: [0.82, 0.80, 0.76, 1.0],
                            roughness: 0.85,
                            metallic: 0.0,
                            ..Default::default()
                        },
                    );
                    storage.set(
                        ground,
                        nightshade_render::components::material::MaterialRef::new(id),
                    );
                }
            });

            camera
        }
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.5;
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
    at: Vec3,
    wanted_reach: Option<f32>,
) -> Framed {
    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("a model did not load: {error}");
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
    let lowest = nightshade_prefab::queries::prefab_bounds(&prefab, &import.meshes)
        .map(|bounds| bounds.0.y)
        .unwrap_or(middle.y);
    let mut spawn = nightshade_prefab::PrefabSpawn { storage, parts };
    nightshade_prefab::queue_model_load(&mut spawn, &mut import);
    let root = nightshade_prefab::spawn_prefab(&mut spawn, &prefab, at);
    if let Some(wanted) = wanted_reach
        && reach > 0.0
        && let Some(transform) = storage.get_mut::<LocalTransform>(root)
    {
        let factor = wanted / reach;
        transform.scale = Vec3::new(factor, factor, factor);
        transform.translation = at - Vec3::new(middle.x, lowest, middle.z) * factor;
    }
    Framed { middle, reach }
}

fn close_for_want_of_a_model(window: &mut nightshade_platform::prelude::Window) {
    window.should_exit = true;
}
