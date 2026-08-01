use crate::resources::{CoinDashResources, Screen};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::systems::controllers::first_person_camera_look_system;
use nightshade_input::{Actions, EngineAction};
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn update(
    resources: Res<CoinDashResources>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    input: Res<Input>,
    actions: Res<Actions<EngineAction>>,
    time: Res<Time>,
    window: Res<Window>,
    cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    if resources.round.screen != Screen::Playing {
        return;
    }
    first_person_camera_look_system(active_camera, input, actions, time, window, cameras);
}
