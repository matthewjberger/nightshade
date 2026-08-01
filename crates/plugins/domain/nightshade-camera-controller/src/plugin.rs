use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::input_capture::InputCapture => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
            nightshade_camera::resources::ActiveCamera => "CameraPlugin",
            nightshade_input::contract::Actions<nightshade_input::contract::EngineAction> => "InputPlugin<EngineAction>",
        ]
    }
    fn build(&self, app: &mut App) {
        register_frame_systems(&mut app.stages);
    }
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    use crate::systems::controllers;
    push_frame_system(stages, Stage::FrameUpdate, controllers::look_camera_system);
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        controllers::touch_fly_camera_system,
    );
    push_frame_system(stages, Stage::FrameUpdate, controllers::fly_camera_system);
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        controllers::pan_orbit_mouse_input,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        controllers::pan_orbit_gamepad_input,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        controllers::pan_orbit_touch_input,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        controllers::pan_orbit_update_transform,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        controllers::apply_camera_requests_system,
    );
}
