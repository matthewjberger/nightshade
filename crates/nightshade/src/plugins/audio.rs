//! Audio playback using the Kira audio engine.
//!
//! Provides components for spatial and non-spatial audio:
//!
//! - [`AudioSource`]: Audio clip playback with volume, looping, and spatial settings
//! - [`AudioListener`]: Marker for the entity that receives spatial audio
//!
//! Audio assets are loaded through the asset system and referenced by name.
//! Spatial audio uses the entity's transform for 3D positioning.
//! The audio feature is optional and requires the `audio` feature flag.
//!
//! # Loading Audio
//!
//! Load audio files and register them with the audio manager:
//!
//! ```ignore
//! let sound_data = load_sound_from_bytes(include_bytes!("../assets/music.ogg"));
//! audio_engine_load_sound(&mut world.resources.audio, "background_music", sound_data);
//! ```
//!
//! # Playing Non-Spatial Audio
//!
//! ```ignore
//! let entity = spawn_entities(world, AUDIO_SOURCE, 1)[0];
//! world.set(entity, AudioSource::new("background_music")
//!     .with_volume(0.8)
//!     .with_looping(true)
//!     .playing());
//! ```
//!
//! # Spatial Audio Setup
//!
//! For 3D positional audio, add an [`AudioListener`] to your camera and use
//! spatial audio sources with transforms:
//!
//! ```ignore
//! // Add listener to camera (typically done once in initialize)
//! world.ecs.worlds[CORE].add_audio_listener(camera_entity);
//! world.set(camera_entity, AudioListener);
//!
//! // Create spatial audio source at a position
//! let source = spawn_entities(world,
//!     AUDIO_SOURCE | LOCAL_TRANSFORM | GLOBAL_TRANSFORM,
//!     1
//! )[0];
//! world.set(source, AudioSource::new("explosion")
//!     .with_spatial(true)
//!     .with_volume(1.0)
//!     .playing());
//! world.set(source, LocalTransform {
//!     translation: Vec3::new(10.0, 0.0, 5.0),
//!     ..Default::default()
//! });
//! ```
//!
//! # WebAssembly Note
//!
//! On WASM, browsers require audio to be initialized after a user interaction
//! (click, touch, or keypress). The audio system handles this automatically,
//! but sounds triggered before any user input will be queued and played once
//! interaction occurs. For best results, trigger initial audio playback in
//! response to user input (e.g., a "Start" button click).

pub mod components;
#[cfg(feature = "audio")]
pub mod resources;
#[cfg(feature = "audio")]
pub mod systems;

pub use components::*;
#[cfg(feature = "audio")]
pub use resources::*;
#[cfg(feature = "audio")]
pub use systems::*;

/// Registers the bus construction and per-frame source sync at the start
/// of the frame, right after the event swap, where they have always run.
#[cfg(feature = "audio")]
pub fn install(world: &mut crate::ecs::world::World) {
    use crate::schedule::{FramePhase, schedule_push};
    let schedule = &mut world.resources.schedules.frame;
    schedule_push(
        schedule,
        FramePhase::Start,
        systems::build_audio_buses_system,
    );
    schedule_push(schedule, FramePhase::Start, systems::update_audio_system);
}

/// Installs the audio engine: on native the device opens at startup, on
/// the web at the first user gesture, and [`install`] adds the audio
/// systems to the frame schedule. Without this plugin the engine never
/// opens the audio device and the frame schedule carries no audio work.
#[cfg(feature = "audio")]
pub struct AudioPlugin;

#[cfg(feature = "audio")]
impl crate::app::Plugin for AudioPlugin {
    fn build(&self, app: &mut crate::app::App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_startup_system(systems::initialize_audio_system);
        #[cfg(target_arch = "wasm32")]
        app.add_system(
            crate::app::Stage::First,
            systems::lazy_initialize_audio_system,
        );
        app.add_startup_system(install);
    }
}
