use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use std::marker::PhantomData;

pub struct ShellOverlayPlugin<C> {
    context: PhantomData<fn() -> C>,
}

impl<C> Default for ShellOverlayPlugin<C> {
    fn default() -> Self {
        Self {
            context: PhantomData,
        }
    }
}

impl<C: Send + Sync + 'static> Plugin for ShellOverlayPlugin<C> {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::input_capture::InputCapture => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world
            .insert_resource(crate::resources::overlay::ShellOverlayState::default());
        app.add_system(
            Stage::Update,
            crate::systems::input::shell_input_system::<C>,
        );
        app.add_system(
            Stage::Update,
            crate::systems::command::shell_command_system::<C>,
        );
        app.add_system(
            Stage::Render,
            crate::systems::overlay::shell_retained_ui_system::<C>,
        );
    }
}
