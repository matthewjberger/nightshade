use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_gltf::queries::sample_model;
use nightshade_graphics::prelude::*;

pub fn initialize(
    mut commands: Commands,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut pending: ResMut<crate::resources::Pending>,
    mut active_camera: ResMut<ActiveCamera>,
    mut command_queues: ResMut<CommandQueues>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut command_queues, &mut render_settings);
    let handle = nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Path(sample_model(theme::MODEL)),
    );
    pending.model = Some(handle);
    let camera = {
        let commands = &mut commands;
        spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_DISTANCE,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(command_queues: &mut CommandQueues, settings: &mut RenderSettings) {
    load_hdr_skybox(&mut *command_queues, theme::SKY.to_vec());
    settings.atmosphere = Atmosphere::Hdr;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.5;
    settings.anti_aliasing = AntiAliasing::Taa;
}

pub fn stand_the_field(
    mut pending: ResMut<crate::resources::Pending>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut field: ResMut<crate::resources::Field>,
    mut window: ResMut<nightshade_platform::prelude::Window>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = pending.model else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut *loading, handle) else {
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
    let each = triangles_in(&import);
    let mut spawn = nightshade_prefab::PrefabSpawn {
        storage,
        parts: parts.reborrow(),
    };
    nightshade_prefab::queue_model_load(&mut spawn, &mut import);
    for index in 0..theme::COPIES {
        nightshade_prefab::spawn_prefab(&mut spawn, &prefab, where_one_stands(index));
    }
    field.copies = theme::COPIES;
    field.triangles = each * theme::COPIES;
}

fn where_one_stands(index: usize) -> Vec3 {
    let across = (index % theme::ACROSS) as f32 - (theme::ACROSS as f32 - 1.0) * 0.5;
    let away = (index / theme::ACROSS) as f32;
    Vec3::new(
        across * theme::SPACING_ACROSS,
        0.0,
        -away * theme::SPACING_AWAY,
    )
}

fn triangles_in(import: &nightshade_prefab::data::ModelImport) -> usize {
    import
        .meshes
        .values()
        .map(|mesh| mesh.indices.len() / 3)
        .sum()
}

fn close_for_want_of_a_model(window: &mut nightshade_platform::prelude::Window) {
    window.should_exit = true;
}
