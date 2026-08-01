use crate::data::ask::Ask;
use crate::data::cell::Cell;
use crate::resources::{Script, Store};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_camera::components::Camera;
use nightshade_camera::resources::ActiveCamera;
use nightshade_picking::queries::{pick_closest_entity, screen_ray};
use nightshade_platform::contract::input::{Input, MouseState};
use nightshade_platform::contract::input_capture::InputCapture;
use nightshade_platform::contract::window::Window;
use nightshade_platform::queries::input_capture::pointer_captured;
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::Visibility;

pub fn reach(
    script: Res<Script>,
    input: Res<Input>,
    input_capture: Res<InputCapture>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    mut store: ResMut<Store>,
    storage: &mut Storage,
) {
    let (Some(pick), Some(picked)) = (script.plan.known.pick, script.plan.known.picked) else {
        return;
    };
    if !input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        return;
    }
    if pointer_captured(&input_capture) {
        return;
    }
    let position = input.mouse.position;
    let Some(ray) = screen_ray(
        &window,
        &viewport,
        &active_camera,
        storage.query_ref::<(&Camera, &GlobalTransform)>(),
        position,
    ) else {
        return;
    };
    let Some(hit) = pick_closest_entity(
        storage.query_ref::<(&BoundingVolume, &GlobalTransform, Option<&Visibility>)>(),
        ray,
    ) else {
        return;
    };
    let Some(row) = store
        .rows
        .iter()
        .position(|held| *held == hit.entity)
        .map(|held| held as u32)
    else {
        return;
    };
    if !store.living(row) || !store.holds(pick, row) {
        return;
    }
    store.asks.push(Ask::SetColumn {
        key: picked,
        row,
        value: Cell::number(1.0),
    });
}
