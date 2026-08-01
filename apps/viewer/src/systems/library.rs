use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_render::resources::capture::ScheduledCapture;

use crate::commands::library::{collect_previews, start_previews};
use crate::data::PREVIEW_PATIENCE;
use crate::resources::{ViewerArguments, ViewerResources};

pub fn draw_previews(mut viewer: ResMut<ViewerResources>, arguments: Res<ViewerArguments>) {
    if arguments.preview {
        return;
    }
    collect_previews(&mut viewer);
    start_previews(&mut viewer);
}

pub fn hold_preview_capture(
    viewer: Res<ViewerResources>,
    arguments: Res<ViewerArguments>,
    time: Res<Time>,
    mut capture: ResMut<ScheduledCapture>,
) {
    if !arguments.preview || time.uptime_milliseconds > PREVIEW_PATIENCE {
        return;
    }
    let pending = if arguments.sky.is_some() {
        !viewer.view.hdr_loaded
    } else {
        viewer.model.roots.is_empty()
    };
    if pending {
        capture.waiting = true;
    }
}
