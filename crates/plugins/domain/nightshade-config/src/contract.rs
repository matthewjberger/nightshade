mod declare;
mod entry;
mod label;
mod store;

pub use entry::{ConfigEntry, ConfigKind, ConfigSchema, ConfigValue};
pub use label::config_label;
pub use store::{Config, options_of};
