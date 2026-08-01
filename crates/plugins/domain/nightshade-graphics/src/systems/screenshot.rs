use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::KeyCode;

pub fn scheduled_capture_system(
    mut capture: ResMut<nightshade_render::resources::capture::ScheduledCapture>,
    mut queues: ResMut<crate::resources::queues::CommandQueues>,
    mut window: ResMut<nightshade_platform::contract::window::Window>,
    loading: Res<crate::resources::loading::LoadingState>,
) {
    let Some(path) = capture.path.clone() else {
        return;
    };
    if capture.waiting {
        return;
    }
    if !crate::queries::loading::loading_pipeline_has_settled(&loading.pipeline, capture.settle) {
        return;
    }
    capture.elapsed += 1;
    let next_frame = capture.frame + nightshade_render::resources::capture::NEXT_PICTURE_AFTER;
    let last_frame = if capture.next.is_some() {
        next_frame
    } else {
        capture.frame
    };
    if !capture.queued && capture.elapsed >= capture.frame {
        crate::commands::queues::capture_screenshot_to_path(&mut queues, path);
        capture.queued = true;
    }
    if capture.elapsed == next_frame
        && let Some(next) = capture.next.clone()
    {
        crate::commands::queues::capture_screenshot_to_path(&mut queues, next);
    }
    if capture.queued
        && capture.exit_after
        && capture.elapsed
            > last_frame + nightshade_render::resources::capture::FRAMES_AFTER_PICTURE
    {
        window.should_exit = true;
    }
}

pub fn screenshot_key_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mut queues: ResMut<crate::resources::queues::CommandQueues>,
) {
    if input.keyboard.just_pressed(KeyCode::PrintScreen) {
        crate::commands::queues::capture_screenshot(&mut queues);
    }
}
