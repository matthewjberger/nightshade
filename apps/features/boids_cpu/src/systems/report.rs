use crate::data::Applied;
use crate::resources::{BoidsResources, StageMark};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

const REPORT_INTERVAL: f32 = 1.0;

pub fn report(
    time: Res<Time>,
    mut resources: ResMut<BoidsResources>,
    applied: Option<ResMut<Applied>>,
) {
    let delta_time = time.delta_time;

    resources.frame_seconds += f64::from(delta_time);
    resources.report_countdown -= delta_time;
    if resources.report_countdown > 0.0 || resources.step_count == 0 {
        return;
    }

    let frames = f64::from(resources.step_count);
    let (imbalance, used, gather, collect, write) = take_phases(applied, frames);

    let resources = &mut *resources;
    let count = resources.boids.len();
    let mean_milliseconds = resources.step_seconds * 1000.0 / frames;
    let mean_sync_milliseconds = resources.sync_seconds * 1000.0 / frames;
    let mean_frame_milliseconds = resources.frame_seconds * 1000.0 / frames;

    resources.reported_milliseconds = mean_milliseconds;
    resources.reported_sync_milliseconds = mean_sync_milliseconds;
    resources.reported_imbalance = imbalance;
    resources.reported_regions = used;
    resources.reported_frame_milliseconds = mean_frame_milliseconds;

    let stages = StageMark::NAMED
        .iter()
        .map(|(stage, name)| {
            format!(
                "{name} {:.2}",
                resources.stage_seconds[*stage as usize] * 1000.0 / frames
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    tracing::info!(
        target: "boids",
        "{count} boids, {used} regions at {imbalance:.2}x peak, frame {mean_frame_milliseconds:.2} ms, flock step {mean_milliseconds:.2} ms, instance sync {mean_sync_milliseconds:.2} ms, elsewhere {:.2} ms, phases gather {gather:.2} collect {collect:.2} write {write:.2} regions {:.2}, stages {stages}, over {} frames",
        mean_frame_milliseconds - mean_milliseconds - mean_sync_milliseconds,
        mean_milliseconds - gather - collect - write,
        resources.step_count
    );

    resources.stage_seconds = [0.0; StageMark::COUNT];

    resources.step_seconds = 0.0;
    resources.sync_seconds = 0.0;
    resources.frame_seconds = 0.0;
    resources.step_count = 0;
    resources.report_countdown = REPORT_INTERVAL;
}

fn take_phases(applied: Option<ResMut<Applied>>, frames: f64) -> (f32, usize, f64, f64, f64) {
    let Some(mut applied) = applied else {
        return (1.0, 1, 0.0, 0.0, 0.0);
    };

    let read = (
        applied.imbalance(),
        applied.regions_used(),
        applied.gather_seconds * 1000.0 / frames,
        applied.collect_seconds * 1000.0 / frames,
        applied.write_seconds * 1000.0 / frames,
    );

    applied.gather_seconds = 0.0;
    applied.collect_seconds = 0.0;
    applied.write_seconds = 0.0;

    read
}
