use crate::contract::PickingOptions;
use crate::queries::{pick_entities, screen_ray};
use crate::resources::{PickWhen, Picked};
use nightshade::ecs::Storage;
use nightshade::prelude::{Res, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_platform::contract::Input;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::contract::input_capture::InputCapture;
use nightshade_platform::contract::window::Window;
use nightshade_platform::queries::input_capture::pointer_captured;
use nightshade_render::resources::viewport::Viewport;

pub fn pick_under_the_pointer(
    when: Res<PickWhen>,
    options: Res<PickingOptions>,
    input: Res<Input>,
    capture: Res<InputCapture>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    mut picked: ResMut<Picked>,
    storage: &mut Storage,
) {
    if *when == PickWhen::Never || pointer_captured(&capture) {
        picked.under = None;
        picked.ray = None;
        return;
    }
    if *when == PickWhen::PointerMoved && !input.mouse.state.contains(MouseState::MOVED) {
        return;
    }
    let ray = screen_ray(
        &window,
        &viewport,
        &active_camera,
        storage.query_ref(),
        input.mouse.position,
    );
    picked.ray = ray;
    let Some(ray) = ray else {
        picked.under = None;
        return;
    };
    picked.under = pick_entities(storage.query_ref(), ray, *options)
        .into_iter()
        .next();
}
