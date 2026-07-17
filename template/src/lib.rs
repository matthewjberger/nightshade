//! Nightshade template.
//!
//! ## Architecture
//!
//! - `src/plugin.rs` — the `TemplatePlugin`. Registers the app member
//!   world, the app resources, and the startup and update systems against
//!   the `App` builder.
//! - `src/ecs.rs` — declares the app member world schema (registered into
//!   the engine group at index `GAME`), the component structs, and the
//!   `TemplateResources` struct for app-wide state.
//! - `src/systems/` — behavior. Each system is a free function with the
//!   shape `fn name(template_resources: &mut TemplateResources, world: &mut World)`.
//!
//! Add a new system by dropping a file in `src/systems/`, registering it
//! in `src/systems.rs`, and pushing it onto a stage in
//! `plugin.rs::TemplatePlugin::build`.

mod ecs;
mod plugin;
mod systems;

pub use plugin::TemplatePlugin;
