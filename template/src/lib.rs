//! Nightshade template.
//!
//! ## Architecture
//!
//! - `src/state.rs` — `Template` struct + `State` trait impl. The state
//!   shell owns your app resources and forwards each lifecycle hook to
//!   system functions.
//! - `src/ecs.rs` — declares the app member world schema (registered
//!   into the engine group at index `GAME`) and the `TemplateResources`
//!   struct for app-wide state.
//! - `src/ecs/components.rs` — component structs.
//! - `src/ecs/resources.rs` — resource structs (app-wide state).
//! - `src/systems/` — behavior. Each system is a free function with the
//!   shape `fn name(template_resources: &mut TemplateResources, world: &mut World)`.
//!
//! Add a new system by dropping a file in `src/systems/`, registering it in
//! `src/systems.rs`, and calling it from `state.rs::run_systems`.

mod ecs;
mod state;
mod systems;

pub use state::Template;
