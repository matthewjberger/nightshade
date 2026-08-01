use nightshade::prelude::*;

use crate::commands::load::{
    finish, load_model, load_model_with_resources, load_sky, spawn_result,
};
use crate::commands::notice::report;
use crate::queries::camera::model_bounds;
use crate::resources::{PendingAsset, ViewerResources};

pub fn poll_load(mut stage: crate::resources::Stage) {
    let pending = stage
        .viewer
        .incoming
        .asset
        .lock()
        .ok()
        .and_then(|mut slot| slot.take());
    let Some(asset) = pending else {
        return;
    };

    match asset {
        PendingAsset::Model(bytes) => load_model(&mut stage, bytes),
        PendingAsset::ModelWithResources {
            gltf,
            resources: files,
        } => load_model_with_resources(&mut stage, gltf, files),
        PendingAsset::Sky(bytes) => {
            load_sky(&mut stage, bytes);
            finish(&mut stage);
        }
    }
}

pub fn flush_report(mut viewer: ResMut<ViewerResources>, storage: &mut nightshade::ecs::Storage) {
    let viewer = &mut *viewer;
    let Some(report) = viewer.model.report.as_mut() else {
        return;
    };
    if report.delay > 0 {
        report.delay -= 1;
        return;
    }
    let dimensions = model_bounds(viewer, storage)
        .map(|(min, max)| {
            let size = max - min;
            [size.x, size.y, size.z]
        })
        .unwrap_or([0.0, 0.0, 0.0]);
    let mut report = viewer.model.report.take().expect("the report is present");
    report.stats.dimensions = dimensions;
    viewer.model.stats = Some(report.stats);
}

pub fn place_the_model(
    mut stage_state: crate::resources::Stage,
    storage: &mut nightshade::ecs::Storage,
) {
    let stage = &mut stage_state;
    let Some(handle) = stage.viewer.model.pending else {
        return;
    };
    let Some(outcome) = nightshade_load::take(stage.loading, handle) else {
        return;
    };
    stage.viewer.model.pending = None;
    match outcome {
        Ok(import) => spawn_result(stage, storage, import),
        Err(error) => {
            let notice = stage.viewer.incoming.notice.clone();
            report(
                &notice,
                format!(
                    "The model did not open.

{error}"
                ),
            );
        }
    }
    finish(stage);
}
