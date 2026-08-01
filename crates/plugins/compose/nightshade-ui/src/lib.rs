pub mod prelude;

pub use commands::resources::{compose, insert_ui_resources};
pub use data::systems::{SystemFn, UiSystemFn, UiSystemPhase, ui_system};
pub use plugin::UiPlugin;
pub use systems::schedule::{default_ui_systems, run_retained_ui_schedule, ui_systems_with};

pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
pub mod systems;

pub const MAX_POPUP_HEIGHT: f32 = 360.0;

pub use commands::builder::*;
pub use commands::scope::*;
pub use commands::widgets::ContextMenuBuilder;
pub use commands::widgets::world_state::{
    ui_is_text_editor, ui_pointer_target, ui_pointer_target_of, ui_wants_keyboard,
    ui_wants_pointer, ui_wants_pointer_of,
};
pub use components::*;
pub use data::state::*;
pub use data::units::*;
pub use resources::*;
