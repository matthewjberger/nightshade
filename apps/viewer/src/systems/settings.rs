use nightshade::prelude::*;

use crate::commands::settings::{apply_settings, keep_settings};
use crate::queries::settings::{current_settings, read_settings};
use crate::resources::{Settings, ViewerArguments, ViewerResources};

const WRITE_DELAY: u32 = 90;

pub fn load_settings(mut stage_state: crate::resources::Stage) {
    let stage = &mut stage_state;
    let preview = stage.arguments.preview;
    let sky = stage.arguments.sky.is_some();
    let settings = if preview {
        Settings {
            grid: false,
            scene_open: false,
            inspector_open: false,
            show_sky: sky,
            ..Settings::default()
        }
    } else {
        read_settings().unwrap_or_default()
    };
    apply_settings(stage, &settings);
    if preview {
        stage.runtime.visible = false;
    }
    let viewer = &mut *stage.viewer;
    viewer.settings = settings;
    viewer.settings_delay = 0;
    viewer.gizmo.enabled = !preview;
}

pub fn save_settings(mut viewer: ResMut<ViewerResources>, arguments: Res<ViewerArguments>) {
    if arguments.preview {
        return;
    }
    let current = current_settings(&viewer);
    if current != viewer.settings {
        viewer.settings = current;
        viewer.settings_delay = WRITE_DELAY;
        return;
    }
    if viewer.settings_delay == 0 {
        return;
    }
    viewer.settings_delay -= 1;
    if viewer.settings_delay == 0 {
        keep_settings(&mut viewer);
    }
}
