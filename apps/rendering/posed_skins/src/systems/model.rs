use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::queries::lights::sun_rotation;

use crate::theme;

#[derive(Default)]
pub struct PendingModel {
    handle: Option<nightshade_load::LoadHandle>,
}

pub fn ask_for_the_model(loading: &mut nightshade_gltf::Loading) -> PendingModel {
    PendingModel {
        handle: Some(nightshade_load::load(
            loading,
            nightshade_load::Source::Url(nightshade_gltf::queries::sample_model_url(theme::MODEL)),
        )),
    }
}

pub fn arrive(
    mut commands: Commands,
    mut pending: ResMut<PendingModel>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut window: ResMut<Window>,
    active_camera: Res<ActiveCamera>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = pending.handle else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut loading, handle) else {
        return;
    };
    pending.handle = None;

    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("{} did not load: {error}", theme::MODEL);
            window.should_exit = true;
            return;
        }
    };
    let Some(prefab) = import.prefabs.first().cloned() else {
        window.should_exit = true;
        return;
    };

    let (middle, reach) = nightshade_prefab::queries::prefab_reach(&prefab, &import.meshes);
    let walking = import
        .animations
        .iter()
        .position(|clip| clip.name.to_lowercase().contains(theme::CLIP));
    let clips = import.animations.clone();
    let skins = import.skins.clone();

    {
        let mut spawn = nightshade_prefab::PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        nightshade_prefab::queue_model_load(&mut spawn, &mut import);
        let entity = nightshade_prefab::spawn_prefab_with_skins(
            &mut spawn,
            &prefab,
            &clips,
            &skins,
            Vec3::zeros(),
        );
        if let Some(index) = walking
            && let Some(player) = spawn.storage.get_mut::<AnimationPlayer>(entity)
        {
            player.play(index);
            player.looping = true;
            player.speed = 1.0;
        }
    }

    if let Some(camera) = active_camera.0 {
        nightshade_camera_controller::commands::frame_pan_orbit_camera(
            storage,
            camera,
            middle,
            reach * theme::CAMERA_BACK,
        );
    }
    stand_the_scene_around_it(&mut commands, reach);
}

fn stand_the_scene_around_it(commands: &mut Commands<'_>, reach: f32) {
    {
        commands.spawn((
            Name("Sun".to_string()),
            LocalTransform {
                translation: Vec3::new(0.0, reach * 4.0, 0.0),
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
            Vec3::new(0.0, reach * -0.02, 0.0),
            Vec3::new(reach * theme::GROUND, reach * 0.02, reach * theme::GROUND),
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
    }
}
