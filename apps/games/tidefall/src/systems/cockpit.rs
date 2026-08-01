use crate::data::{Ask, HEADLIGHT};
use crate::resources::TidefallResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_render::prelude::Light;

pub fn hold_the_window(input: Res<Input>, mut window: ResMut<Window>) {
    if window.cursor_locked && !window.is_focused {
        set_cursor_locked(&mut window, false);
        set_cursor_visible(&mut window, true);
        return;
    }
    if !window.cursor_locked
        && window.is_focused
        && input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
    {
        set_cursor_locked(&mut window, true);
        set_cursor_visible(&mut window, false);
    }
}

pub fn switch_the_lights(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<TidefallResources>,
    mut lamps: Query<(&mut Light,)>,
) {
    if !actions.just_pressed(Ask::Lights) {
        return;
    }
    resources.lights_on = !resources.lights_on;
    let intensity = if resources.lights_on { HEADLIGHT } else { 0.0 };
    for entity in resources.headlights.clone() {
        lamps.for_entity(entity, |(light,)| light.intensity = intensity);
    }
}
