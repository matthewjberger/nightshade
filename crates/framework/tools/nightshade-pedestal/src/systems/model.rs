use crate::resources::{Pedestal, PendingModel};
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_platform::contract::window::Window;

pub fn arrive(
    mut pending_model: ResMut<PendingModel>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    pedestal: Res<Pedestal>,
    mut window: ResMut<Window>,
    active_camera: Res<ActiveCamera>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = pending_model.handle else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut *loading, handle) else {
        return;
    };
    pending_model.handle = None;

    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            let model = pedestal.model;
            tracing::error!("{model} did not load: {error}");
            window.should_exit = true;
            return;
        }
    };
    let Some(prefab) = import.prefabs.first().cloned() else {
        window.should_exit = true;
        return;
    };

    let (middle, reach) = nightshade_prefab::queries::prefab_reach(&prefab, &import.meshes);
    {
        let mut spawn = nightshade_prefab::PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        nightshade_prefab::queue_model_load(&mut spawn, &mut import);
        nightshade_prefab::spawn_prefab(&mut spawn, &prefab, Vec3::zeros());
    }
    let Some(camera) = active_camera.0 else {
        return;
    };
    let back = pedestal.back;
    nightshade_camera_controller::commands::frame_pan_orbit_camera(
        storage,
        camera,
        middle,
        reach * back,
    );
}
