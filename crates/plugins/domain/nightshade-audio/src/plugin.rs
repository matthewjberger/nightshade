use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world
            .insert_resource(crate::resources::AudioEngine::default());
        app.world
            .insert_resource(crate::resources::Sounds::default());
        #[cfg(not(target_arch = "wasm32"))]
        app.add_system(
            Stage::Startup,
            crate::systems::lifecycle::initialize_audio_system,
        );
        #[cfg(target_arch = "wasm32")]
        app.add_system(
            Stage::First,
            crate::systems::lifecycle::lazy_initialize_audio_system,
        );
        register_frame_systems(&mut app.stages);
    }
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::FrameStart,
        crate::systems::requests::drain_audio_requests_system,
    );
    push_frame_system(
        stages,
        Stage::FrameStart,
        crate::systems::loading::collect_the_sounds,
    );
    #[cfg(not(target_arch = "wasm32"))]
    {
        push_frame_system(
            stages,
            Stage::FrameStart,
            crate::systems::buses::build_audio_buses_system,
        );
        push_frame_system(
            stages,
            Stage::FrameStart,
            crate::systems::sources::sync_audio_listener_system,
        );
        push_frame_system(
            stages,
            Stage::FrameStart,
            crate::systems::sources::update_audio_sources_system,
        );
        push_frame_system(
            stages,
            Stage::FrameStart,
            crate::systems::sources::sync_audio_pause_system,
        );
        push_frame_system(
            stages,
            Stage::FrameStart,
            crate::systems::acoustics::drive_audio_acoustics_system,
        );
    }
}
