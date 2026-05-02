//! Nightshade template.
//!
//! ## Architecture
//!
//! - `src/state.rs` — `Template` struct + `State` trait impl. The state shell
//!   owns your user-side ECS world and forwards each lifecycle hook to
//!   system functions.
//! - `src/ecs.rs` — declares the `TemplateWorld` (a [`freecs`] world) with
//!   your components, tags, events, and resources.
//! - `src/ecs/components.rs` — component structs.
//! - `src/ecs/resources.rs` — resource structs (app-wide state).
//! - `src/systems/` — behavior. Each system is a free function with the
//!   shape `fn name(template_world: &mut TemplateWorld, world: &mut World)`.
//!
//! Add a new system by dropping a file in `src/systems/`, registering it in
//! `src/systems.rs`, and calling it from `state.rs::run_systems`.

mod ecs;
mod state;
mod systems;

pub use state::Template;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: nightshade::prelude::AndroidApp) {
    nightshade::prelude::launch_android(app, Template::default()).unwrap();
}
