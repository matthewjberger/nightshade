use crate::components::NodePlace;
use crate::data::Holding;
use crate::resources::{GraphGrip, GraphView};
use nightshade::ecs::system_param::{Query, Res, ResMut};

pub fn move_what_is_held(
    grip: Res<GraphGrip>,
    mut view: ResMut<GraphView>,
    mut places: Query<(&mut NodePlace,)>,
) {
    let step = grip.step;
    match grip.holding {
        Holding::View => view.pan += step,
        Holding::Node(node) => places.for_entity(node, |(place,)| {
            place.0 += step;
        }),
        Holding::Nothing | Holding::Note(_) | Holding::Inlet(_) | Holding::Outlet(_) => {}
    }
}
