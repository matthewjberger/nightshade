use crate::data::Ask;
use crate::resources::Walk;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_input::Actions;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_platform::prelude::*;

pub fn toggle_the_walk(
    actions: Res<Actions<Ask>>,
    mut walk: ResMut<Walk>,
    mut active_camera: ResMut<ActiveCamera>,
    mut window: ResMut<Window>,
    mut controllers: Query<(&mut CharacterControllerComponent,)>,
) {
    if actions.just_pressed(Ask::ToggleWalk) {
        walk.wanted = Some(!walk.walking);
    }
    let Some(walking) = walk.wanted.take() else {
        return;
    };
    let (Some(orbit), Some(body), Some(eyes)) = (walk.orbit, walk.body, walk.eyes) else {
        return;
    };
    walk.walking = walking;
    active_camera.0 = Some(if walking { eyes } else { orbit });
    set_cursor_locked(&mut window, walking);
    set_cursor_visible(&mut window, !walking);
    controllers.for_entity(body, |(controller,)| {
        controller.engine_input_enabled = walking;
        controller.velocity = Vec3::zeros();
    });
}
