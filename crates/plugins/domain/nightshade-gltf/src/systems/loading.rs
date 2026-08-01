use crate::data::LOAD_LIMIT;
use crate::resources::Loading;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::ResMut;
use nightshade_load::{gather, in_flight, take};
use nightshade_prefab::commands::{queue_model_load, spawn_prefab};
use nightshade_prefab::resources::{PrefabParts, PrefabSpawn};
use nightshade_render::resources::capture::ScheduledCapture;

pub fn collect_the_models(
    mut loading: ResMut<Loading>,
    mut capture: ResMut<ScheduledCapture>,
    mut parts: PrefabParts,
    storage: &mut Storage,
) {
    let landed = gather(&mut loading, LOAD_LIMIT);
    let reading = in_flight(&loading) > 0 || !landed.is_empty();
    capture.waiting = reading;
    for (handle, wanted) in landed {
        let Some(position) = wanted else {
            continue;
        };
        let Some(outcome) = take(&mut loading, handle) else {
            continue;
        };
        let mut import = match outcome {
            Ok(import) => import,
            Err(error) => {
                tracing::error!("a model did not load: {error}");
                continue;
            }
        };
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        for prefab in &import.prefabs {
            spawn_prefab(&mut spawn, prefab, position);
        }
    }
}
