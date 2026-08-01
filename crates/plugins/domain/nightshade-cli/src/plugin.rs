use crate::commands;
use crate::data::DEFAULT_CAPTURE_FRAME;
use crate::queries;
use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_platform::contract::window::Window;
use nightshade_render::resources::capture::ScheduledCapture;

pub struct CliPlugin<T> {
    pub capture_frame: u64,
    pub arguments: std::marker::PhantomData<T>,
}

impl<T> Default for CliPlugin<T> {
    fn default() -> Self {
        Self {
            capture_frame: DEFAULT_CAPTURE_FRAME,
            arguments: std::marker::PhantomData,
        }
    }
}

impl<T> Plugin for CliPlugin<T>
where
    T: clap::Args + Default + Send + Sync + 'static,
{
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::resources::capture::ScheduledCapture => "GraphicsPlugin",
        ]
    }

    fn build(&self, app: &mut App) {
        assert!(
            queries::composition::window_plugin_composed(&app.world),
            "CliPlugin writes the window and the scheduled capture, so compose it after WindowPlugin"
        );
        let arguments = queries::arguments::parse::<T>();
        commands::capture::schedule_capture(
            app.world.res_mut::<ScheduledCapture>(),
            &arguments.capture,
            self.capture_frame,
        );
        commands::window::size_window(app.world.res_mut::<Window>(), &arguments.window);
        app.insert_resource(arguments.app);
    }
}
