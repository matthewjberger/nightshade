pub mod commands;
pub mod data;
#[cfg(feature = "plugin")]
pub mod plugin;
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

pub use data::{Save, SaveCodec, SaveDue, SaveHeader};
#[cfg(feature = "plugin")]
pub use plugin::SavePlugin;
#[cfg(feature = "plugin")]
pub use resources::Autosave;

pub mod prelude {
    pub use crate::commands::{save_remove, save_write};
    pub use crate::data::{Save, SaveCodec, SaveDue, SaveHeader};
    #[cfg(feature = "plugin")]
    pub use crate::plugin::SavePlugin;
    pub use crate::queries::{save_header, save_read, save_slots};
    #[cfg(feature = "plugin")]
    pub use crate::resources::Autosave;
}
