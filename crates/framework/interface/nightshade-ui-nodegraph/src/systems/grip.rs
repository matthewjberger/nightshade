use crate::data::Holding;
use crate::queries::what_is_under;
use crate::resources::{GraphGrip, GraphView, NodeGraphSettings};
use nalgebra_glm::Vec2;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_ui::prelude::{UiLayoutNode, UiNodeInteraction};

pub fn take_the_grip(
    settings: Res<NodeGraphSettings>,
    view: Res<GraphView>,
    input: Res<Input>,
    mut grip: ResMut<GraphGrip>,
    storage: &mut Storage,
) {
    let Some(canvas) = view.canvas else {
        grip.inside = false;
        return;
    };

    let origin = storage
        .get::<UiLayoutNode>(canvas)
        .map(|node| node.computed_rect.min)
        .unwrap_or_default();
    let local = input.mouse.position - origin;
    grip.step = if grip.tracking {
        local - grip.local
    } else {
        Vec2::zeros()
    };
    grip.local = local;
    grip.pointer = local - view.pan;
    grip.tracking = true;

    let (hovered, pressed, right_clicked) = storage
        .get::<UiNodeInteraction>(canvas)
        .map(|found| (found.hovered, found.pressed, found.right_clicked))
        .unwrap_or((false, false, false));

    grip.inside = hovered;
    grip.cutting = right_clicked;
    grip.over = if hovered {
        what_is_under(storage, &settings, grip.pointer)
    } else {
        Holding::Nothing
    };

    if matches!(grip.holding, Holding::Nothing) {
        if pressed && hovered {
            grip.holding = match grip.over {
                Holding::Nothing => Holding::View,
                found => found,
            };
        }
        return;
    }

    grip.letting_go = !pressed;
}
