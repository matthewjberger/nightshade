use crate::resources::{BlackmarrowResources, Console, Screen};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::systems::controllers::first_person_camera_look_system;
use nightshade_input::{Actions, EngineAction};
use nightshade_platform::prelude::*;
use nightshade_render::resources::capture::ScheduledCapture;
use nightshade_scene::prelude::*;
use nightshade_shell::{ShellState, shell_is_capturing_input};

pub fn update(
    resources: Res<BlackmarrowResources>,
    capture: Res<ScheduledCapture>,
    active_camera: Res<ActiveCamera>,
    input: Res<Input>,
    actions: Res<Actions<EngineAction>>,
    time: Res<Time>,
    window: Res<Window>,
    shell: Res<ShellState<Console>>,
    cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    if resources.screen != Screen::Playing
        || capture.path.is_some()
        || shell_is_capturing_input(&shell)
    {
        return;
    }
    first_person_camera_look_system(active_camera, input, actions, time, window, cameras);
}
