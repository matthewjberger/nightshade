use crate::data::CaptureArguments;
use nightshade_render::resources::capture::{QUIET_FRAMES, ScheduledCapture};

pub fn schedule_capture(
    capture: &mut ScheduledCapture,
    arguments: &CaptureArguments,
    default_frame: u64,
) {
    capture.path = arguments.capture.clone();
    capture.next = arguments.capture_next.clone();
    capture.frame = arguments.capture_frame.unwrap_or(default_frame);
    capture.settle = arguments.capture_settle.unwrap_or(QUIET_FRAMES);
    capture.exit_after = true;
    capture.elapsed = 0;
    capture.queued = false;
}
