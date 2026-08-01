use crate::data::StateTransition;
use crate::queries::{current_state, in_state};
use crate::resources::{NextState, State};
use nightshade::ecs::ResourceHost;
use nightshade::ecs::Schedule;
use nightshade::ecs::gate::{IntoGroupRunner, run_if};
use nightshade::ecs::system_param::EventHost;

pub fn apply_state_transition<S, W>(host: &mut W)
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost + EventHost,
{
    let transition = {
        let map = host.resource_map_mut();
        let Some(state) = map.get::<State<S>>() else {
            return;
        };
        let current = state.current;
        let initialized = state.initialized;

        if !initialized {
            map.get_mut::<State<S>>()
                .expect("state present above")
                .initialized = true;
            if let Some(next) = map.get_mut::<NextState<S>>()
                && next.pending == Some(current)
            {
                next.pending = None;
            }
            Some(StateTransition {
                before: None,
                after: current,
            })
        } else {
            let pending = map.get::<NextState<S>>().and_then(|next| next.pending);
            match pending {
                Some(next) => {
                    if let Some(next_state) = map.get_mut::<NextState<S>>() {
                        next_state.pending = None;
                    }
                    if next != current {
                        map.get_mut::<State<S>>()
                            .expect("state present above")
                            .current = next;
                        Some(StateTransition {
                            before: Some(current),
                            after: next,
                        })
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
    };
    if let Some(transition) = transition {
        host.event_bus_mut().send::<StateTransition<S>>(transition);
    }
}

pub fn while_in<S, W, Marker>(
    state: S,
    systems: impl IntoGroupRunner<W, Marker>,
) -> impl FnMut(&mut W) + Send + 'static
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    run_if(move |world: &W| in_state::<S, W>(world, state), systems)
}

pub fn while_in_any<S, W, Marker>(
    states: impl IntoIterator<Item = S>,
    systems: impl IntoGroupRunner<W, Marker>,
) -> impl FnMut(&mut W) + Send + 'static
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    let states: Vec<S> = states.into_iter().collect();
    run_if(
        move |world: &W| states.contains(&current_state::<S, W>(world)),
        systems,
    )
}

pub fn on_enter<S, W, Marker>(
    state: S,
    systems: impl IntoGroupRunner<W, Marker>,
) -> impl FnMut(&mut W) + Send + 'static
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: EventHost + 'static,
{
    let mut cursor = 0u64;
    let mut runner = systems.into_group_runner();
    move |host: &mut W| {
        let entered = host
            .event_bus_mut()
            .consume::<StateTransition<S>>(&mut cursor)
            .iter()
            .any(|transition| transition.after == state);
        if entered {
            runner(host);
        }
    }
}

pub fn on_exit<S, W, Marker>(
    state: S,
    systems: impl IntoGroupRunner<W, Marker>,
) -> impl FnMut(&mut W) + Send + 'static
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: EventHost + 'static,
{
    let mut cursor = 0u64;
    let mut runner = systems.into_group_runner();
    move |host: &mut W| {
        let exited = host
            .event_bus_mut()
            .consume::<StateTransition<S>>(&mut cursor)
            .iter()
            .any(|transition| transition.before == Some(state));
        if exited {
            runner(host);
        }
    }
}

pub trait StateScheduleExt<W> {
    fn add_state_transitions<S>(&mut self, name: &'static str) -> &mut Self
    where
        S: Copy + PartialEq + Send + Sync + 'static;
}

impl<W: ResourceHost + EventHost + 'static> StateScheduleExt<W> for Schedule<W> {
    fn add_state_transitions<S>(&mut self, name: &'static str) -> &mut Self
    where
        S: Copy + PartialEq + Send + Sync + 'static,
    {
        self.push(name, apply_state_transition::<S, W>)
    }
}
