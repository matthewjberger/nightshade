//! The engine's capability plugins, each a self-contained unit: the plugin
//! struct plus the systems, components, and resources its feature owns.
//! They are shaped so any of them could stand alone as a crate, and are
//! built into this crate behind feature flags on purpose: cargo features
//! decide what code exists, composing the plugin is what activates it.

pub mod audio;
pub mod camera_controller;
pub mod cutscene;
#[cfg(feature = "egui")]
pub mod egui;
pub mod exit_on_escape;
#[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
pub mod file_watcher;
#[cfg(feature = "gamepad")]
pub mod gamepad;
#[cfg(feature = "gizmos")]
pub mod gizmos;
pub mod log;
pub mod navmesh;
pub mod physics;
pub mod render;
#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
pub mod steam;
#[cfg(feature = "terrain")]
pub mod terrain;
pub mod ui;
