use nightshade::prelude::{App, Plugin};

#[derive(Default)]
pub struct OpenXrPlugin {
    pub z_near: Option<f32>,
}

impl Plugin for OpenXrPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
            nightshade_camera::resources::ActiveCamera => "CameraPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        use nightshade::prelude::Stage;

        app.add_plugin(nightshade_wgpu::RenderPlugin::default());

        app.insert_resource(crate::contract::XrSettings {
            z_near: self.z_near.unwrap_or(0.05),
            eye_resolution: None,
        });
        app.insert_resource(crate::contract::XrRigSlot::default());
        app.insert_resource(crate::contract::XrFrame::default());
        app.insert_resource(crate::contract::XrInput::default());
        app.insert_resource(crate::contract::XrUiPanel::default());
        app.insert_resource(crate::contract::XrUiPointer::default());

        app.add_system(
            Stage::Startup,
            (
                crate::systems::frame::apply_headset_settings,
                crate::systems::frame::spawn_xr_rig,
            ),
        );
        app.add_system(Stage::First, crate::systems::frame::xr_pose_system);
        app.add_system(Stage::First, crate::systems::ui_pointer::ui_pointer_system);
        app.add_system(Stage::Last, crate::systems::ui_cursor::ui_cursor_system);
        app.stages.stage_mut(Stage::Last.name()).order(
            std::any::type_name_of_val(&nightshade_ui::systems::render_sync::publish_frame),
            std::any::type_name_of_val(&crate::systems::ui_cursor::ui_cursor_system),
        );

        app.take_over_runner("OpenXrPlugin", crate::runner::run_openxr);
    }
}
