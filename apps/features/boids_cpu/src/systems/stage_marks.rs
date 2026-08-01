use crate::resources::{BoidsResources, StageMark};
use nightshade::prelude::*;

pub fn after_first(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::First);
}

pub fn after_update(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::Update);
}

pub fn after_frame_start(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::FrameStart);
}

pub fn after_frame_update(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::FrameUpdate);
}

pub fn after_frame_post_update(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::FramePostUpdate);
}

pub fn after_render(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::Render);
}

pub fn after_last(mut resources: ResMut<BoidsResources>) {
    mark(&mut resources, StageMark::Last);
}

fn mark(resources: &mut BoidsResources, stage: StageMark) {
    let now = Instant::now();
    if let Some(previous) = resources.stage_clock {
        resources.stage_seconds[stage as usize] += now.duration_since(previous).as_secs_f64();
    }
    resources.stage_clock = Some(now);
}
