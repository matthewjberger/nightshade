//! Nightshade template.
//!
//! ## Architecture
//!
//! - `src/plugin.rs` — the `TemplatePlugin`: inserts the app resources and
//!   registers the systems against the `App` builder. Registration only, no
//!   behavior.
//! - `src/ecs.rs` — the app member world schema (registered into the engine
//!   group at index `GAME`) and the `TemplateResources` aggregate; the
//!   component structs live in `src/ecs/components.rs`, the resource structs
//!   in `src/ecs/resources.rs`.
//! - `src/systems/` — behavior, one file per concern: `setup::initialize`
//!   builds the opening scene, `example::tick` runs each frame. A system is a
//!   free function taking `&mut World`, optionally preceded by
//!   `&mut TemplateResources` for app state.
//!
//! Add a system by dropping a file in `src/systems/`, declaring it in
//! `src/systems.rs`, and registering it on a stage in
//! `plugin.rs::TemplatePlugin::build`.

mod ecs;
mod plugin;
mod systems;

pub use plugin::TemplatePlugin;
