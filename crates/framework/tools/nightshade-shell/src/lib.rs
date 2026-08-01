pub mod data;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use plugin::ShellOverlayPlugin;
pub use resources::{CommandRegistry, ShellOverlay, ShellOverlayState, ShellState};
pub use systems::input::{shell_is_capturing_input, shell_key};
