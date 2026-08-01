use crate::resources::State;
use nightshade::ecs::ResourceHost;

pub fn current_state<S, W>(host: &W) -> S
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    host.resource_map()
        .get::<State<S>>()
        .unwrap_or_else(|| {
            panic!(
                "no state of type {} was inserted; compose StatePlugin first",
                std::any::type_name::<S>()
            )
        })
        .current
}

pub fn in_state<S, W>(host: &W, state: S) -> bool
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    current_state::<S, W>(host) == state
}
