//! The window plugin: opens a visible window and installs the winit event
//! loop as the app runner.

use crate::app::{App, Plugin};

/// Adds the winit event loop and a visible window.
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
    fn build(&self, app: &mut App) {
        app.add_plugin_if_absent(crate::plugins::core::CorePlugin);
        app.world.res_mut::<crate::platform::window::Window>().title = self.title.clone();
        if self.size.is_some() {
            app.world
                .res_mut::<crate::platform::window::Window>()
                .initial_size = self.size;
        }
        app.world
            .res_mut::<crate::platform::window::Window>()
            .start_hidden = false;
        app.set_runner_if_unset(run_windowed);
    }
}

/// Drives a composed [`App`] through the winit event loop: the startup
/// schedule once, then the frame stages every frame. Installed as the app
/// runner by the window and renderer plugins.
pub(crate) fn run_windowed(mut app: App) -> Result<(), Box<dyn std::error::Error>> {
    let log_guards = app.take_log_guards();
    let (world, state) = app.into_parts();
    let result = crate::run::launch_with_world(state, world);
    drop(log_guards);
    result
}
