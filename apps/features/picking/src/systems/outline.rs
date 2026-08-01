use crate::resources::PickingResources;
use nightshade::prelude::*;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_render::resources::outline::Outline;

pub fn apply_click(
    mut picking: ResMut<PickingResources>,
    input: Res<Input>,
    capture: Res<InputCapture>,
) {
    let picking = &mut *picking;
    if !input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        return;
    }
    if pointer_captured(&capture) {
        return;
    }
    picking.selected_entity = match (picking.selected_entity, picking.highlighted_entity) {
        (Some(previous), Some(now)) if previous == now => None,
        (_, Some(now)) => Some(now),
        (_, None) => None,
    };
}

pub fn sync_outline(picking: Res<PickingResources>, mut outline: ResMut<Outline>) {
    outline.active_entity = picking.highlighted_entity.or(picking.selected_entity);
}
