//! Nightshade template.
//!
//! ## Architecture
//!
//! - `src/plugin.rs` — the `TemplatePlugin`: inserts the app resources and
//!   registers the systems against the `App` builder. Registration only, no
//!   behavior.
//! - `src/components.rs` — the game components and the `dynamic_schema!` that
//!   registers them into the app member world (engine group index `GAME`).
//! - `src/resources.rs` — the `TemplateResources` aggregate and the resource
//!   structs for app-wide state.
//! - `src/systems/` — behavior, one file per concern: `setup::initialize`
//!   builds the scene and spawns a spinning cube, `example::tick` runs each
//!   frame. A system is a free function taking `&mut World`, optionally
//!   preceded by `&mut TemplateResources` for app state.
//!
//! Add a system by dropping a file in `src/systems/`, declaring it in
//! `src/systems.rs`, and registering it on a stage in
//! `plugin.rs::TemplatePlugin::build`.

mod components;
mod plugin;
mod resources;
mod systems;

pub use plugin::TemplatePlugin;
