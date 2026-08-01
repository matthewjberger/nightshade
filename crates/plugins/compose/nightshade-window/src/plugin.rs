use nightshade::app::App;
use nightshade::plugin::Plugin;

pub struct WindowPlugin {
    pub title: String,
    pub size: Option<(u32, u32)>,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            title: "Nightshade".to_string(),
            size: None,
        }
    }
}

impl Plugin for WindowPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::input_capture::InputCapture => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_platform::PlatformPlugin);
        app.add_plugin(nightshade_scene::ScenePlugin);
        app.add_plugin(nightshade_input::InputPlugin::<
            nightshade_input::EngineAction,
        >::new());
        app.add_plugin(nightshade_graphics::GraphicsPlugin);
        app.world
            .resource_or_insert_default::<crate::resources::PendingHost>();
        app.world
            .res_mut::<nightshade_platform::contract::window::Window>()
            .title = self.title.clone();
        if self.size.is_some() {
            app.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .initial_size = self.size;
        }
        app.world
            .res_mut::<nightshade_platform::contract::window::Window>()
            .start_hidden = false;
        app.add_system(
            nightshade::app::Stage::First,
            crate::run::file_drop::process_dropped_files_system,
        );
        #[cfg(target_arch = "wasm32")]
        app.add_system(
            nightshade::app::Stage::FrameUpdate,
            crate::systems::canvas::sync_wasm_canvas_size,
        );
    }
}
