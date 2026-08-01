use nightshade::prelude::*;
use nightshade_camera::components::ViewportShading;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;

use crate::commands::load::load_default;
use crate::commands::render::point_sun;
use crate::theme::{CLEAR_COLOR, OUTLINE_COLOR};

pub fn initialize(
    mut stage_state: crate::resources::Stage,
    mut commands: nightshade::prelude::Commands,
    _storage: &mut nightshade::ecs::Storage,
) {
    let stage = &mut stage_state;
    {
        let settings = &mut *stage.settings;
        settings.atmosphere = Atmosphere::Sky;
        settings.clear_color = CLEAR_COLOR;
        settings.ssao_enabled = true;
        settings.bloom_enabled = true;
        settings.show_sky = true;
    }
    stage.debug_draw.show_grid = true;
    stage.outline.color = OUTLINE_COLOR;

    let (intensity, shadows, yaw, pitch) = {
        let viewer = &mut *stage.viewer;
        (
            viewer.sun.intensity,
            viewer.sun.shadows,
            viewer.sun.yaw,
            viewer.sun.pitch,
        )
    };
    let (sun, camera) = {
        let commands = &mut commands;
        {
            let sun = spawn_sun(commands);
            commands.queue(move |storage, _| {
                if let Some(light) = storage.get_mut::<Light>(sun) {
                    light.cast_shadows = shadows;
                    light.intensity = intensity;
                    light.shadow_bias = 0.008;
                }
                point_sun(storage, sun, yaw, pitch);
            });
            let camera = spawn_pan_orbit_camera_at(
                commands,
                Vec3::zeros(),
                4.0,
                0.7,
                0.5,
                "Viewer Camera".to_string(),
            );
            commands.set(camera, ViewportShading::default());
            (sun, camera)
        }
    };
    stage.active_camera.0 = Some(camera);

    load_default(stage);

    let viewer = &mut *stage.viewer;
    viewer.sun.entity = Some(sun);
    viewer.camera_input.viewer_camera = Some(camera);
    viewer.view.render_panel_dirty = true;
    viewer.view.tree_dirty = true;
}
