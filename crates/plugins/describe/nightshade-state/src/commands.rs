use crate::resources::{NextState, State};
use nightshade::ecs::ResourceHost;

pub fn insert_state<S, W>(host: &mut W, initial: S)
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    let map = host.resource_map_mut();
    map.insert(State {
        current: initial,
        initialized: false,
    });
    map.insert(NextState::<S> { pending: None });
}

pub fn next_state<S, W>(host: &mut W, state: S)
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    host.resource_map_mut()
        .get_mut::<NextState<S>>()
        .unwrap_or_else(|| {
            panic!(
                "no state of type {} was inserted; compose StatePlugin first",
                std::any::type_name::<S>()
            )
        })
        .pending = Some(state);
}
