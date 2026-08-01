use crate::commands::audio_engine_initialize;
use crate::queries::audio_engine_is_initialized;
use crate::resources::AudioEngine;
#[cfg(target_arch = "wasm32")]
use nightshade::ecs::system_param::Res;
use nightshade::ecs::system_param::ResMut;
use tracing;

#[cfg(not(target_arch = "wasm32"))]
pub fn initialize_audio_system(mut engine: ResMut<AudioEngine>) {
    if audio_engine_is_initialized(&engine) {
        return;
    }
    if let Err(error) = audio_engine_initialize(&mut engine) {
        tracing::error!("Failed to initialize audio: {}", error);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn lazy_initialize_audio_system(
    mut engine: ResMut<AudioEngine>,
    input: Res<nightshade_platform::contract::input::Input>,
) {
    if audio_engine_is_initialized(&engine) {
        return;
    }
    let gestured = input.events.iter().any(|event| {
        matches!(
            event,
            nightshade_platform::contract::AppEvent::Keyboard { .. }
                | nightshade_platform::contract::AppEvent::Mouse { .. }
        )
    });
    if !gestured {
        return;
    }
    if let Err(error) = audio_engine_initialize(&mut engine) {
        tracing::error!("Failed to initialize audio (WASM): {}", error);
    }
}
