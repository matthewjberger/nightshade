//! The engine's capability plugins, each a self-contained unit: the plugin
//! struct plus the systems, components, and resources its feature owns.
//! They are shaped so any of them could stand alone as a crate, and are
//! built into this crate behind feature flags on purpose: cargo features
//! decide what code exists, composing the plugin is what activates it.

pub mod camera_controller;
pub mod core;
pub mod cutscene;
pub mod exit_on_escape;
#[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
pub mod file_watcher;
#[cfg(feature = "gamepad")]
pub mod gamepad;
pub mod log;
pub mod physics;
pub mod render;
#[cfg(feature = "terrain")]
pub mod terrain;
pub mod ui;
pub mod window;
