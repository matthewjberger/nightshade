//! An optional state machine over the dynamic layer, behind the `state`
//! feature: a current-and-next value per state type, transitions that emit an
//! event, run-condition gating of systems, and enter and exit combinators.
//! The state type is yours; the machinery combines the run conditions and
//! system parameters of the ECS layer with a small state model, all in this
//! one optional feature.
//!
//! A state type is any `Copy + PartialEq` value, usually an enum. Insert one
//! with [`insert_state`], drive it with [`apply_state_transition`] pushed onto
//! an early stage (or [`add_state_transitions`](StateScheduleExt::add_state_transitions)),
//! and request transitions with [`next_state`]. Each applied transition sends
//! a [`StateTransition`] event, so [`on_enter`] and [`on_exit`] read it for
//! exactly-once enter and exit logic, and [`while_in`], [`while_in_any`], and
//! [`run_if`] gate systems on the current state. The state value itself is
//! plain data in the host's resource map, so nothing here reaches past the
//! host it is given.
//!
//! ```rust
//! use nightshade_ecs::Schedule;
//! use nightshade_ecs::dynamic::DynWorld;
//! use nightshade_ecs::system_param::{ResMut, ScheduleExt};
//! use nightshade_ecs::state::{StateScheduleExt, insert_state, next_state, while_in};
//!
//! #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//! enum Screen {
//!     Title,
//!     Playing,
//! }
//!
//! struct Ticks(u32);
//!
//! fn tick(mut ticks: ResMut<Ticks>) {
//!     ticks.0 += 1;
//! }
//!
//! let mut world = DynWorld::new();
//! insert_state(&mut world, Screen::Title);
//! world.insert_resource(Ticks(0));
//!
//! let mut schedule = Schedule::new();
//! schedule.add_state_transitions::<Screen>("screen_transitions");
//! schedule.push("tick", while_in(Screen::Playing, tick));
//!
//! schedule.run(&mut world); // enters Title; the gated tick is skipped
//! assert_eq!(world.resource::<Ticks>().unwrap().0, 0);
//!
//! next_state(&mut world, Screen::Playing);
//! schedule.run(&mut world); // transitions to Playing, then tick runs
//! assert_eq!(world.resource::<Ticks>().unwrap().0, 1);
//! ```

use crate::Schedule;
use crate::dynamic::ResourceHost;
use crate::system_param::{EventHost, IntoSystem};
use std::marker::PhantomData;

/// The current value of state type `S`, a resource inserted by
/// [`insert_state`]. Read it with [`current_state`] or [`in_state`].
/// `initialized` is false until the first [`apply_state_transition`] runs the
/// initial entry.
pub struct State<S> {
    pub current: S,
    pub initialized: bool,
}

/// The pending transition for state type `S`, a resource inserted by
/// [`insert_state`]. Request a transition with [`next_state`]; it lands on the
/// next [`apply_state_transition`].
pub struct NextState<S> {
    pub pending: Option<S>,
}

/// The event [`apply_state_transition`] sends each time state type `S`
/// changes. `before` is `None` on the initial entry, otherwise the state that
/// was left. Read it with an
/// [`EventReader`](crate::system_param::EventReader), or through [`on_enter`]
/// and [`on_exit`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateTransition<S> {
    pub before: Option<S>,
    pub after: S,
}

/// Inserts the [`State`] and [`NextState`] resources for `S`, starting at
/// `initial`. The initial state's entry runs on the first
/// [`apply_state_transition`], which also emits its [`StateTransition`].
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

/// The current state of type `S`. Panics if [`insert_state`] was never called
/// for `S`.
pub fn current_state<S, W>(host: &W) -> S
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    host.resource_map()
        .get::<State<S>>()
        .unwrap_or_else(|| {
            panic!(
                "no state of type {} was inserted; call insert_state first",
                std::any::type_name::<S>()
            )
        })
        .current
}

/// Whether the current state of type `S` is `state`.
pub fn in_state<S, W>(host: &W, state: S) -> bool
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    current_state::<S, W>(host) == state
}

/// Requests a transition to `state`, applied on the next
/// [`apply_state_transition`]. A request for the current state is dropped
/// without emitting a transition. Panics if [`insert_state`] was never called
/// for `S`.
pub fn next_state<S, W>(host: &mut W, state: S)
where
    S: Copy + PartialEq + Send + Sync + 'static,
    W: ResourceHost,
{
    host.resource_map_mut()
        .get_mut::<NextState<S>>()
        .unwrap_or_else(|| {
            panic!(
                "no state of type {} was inserted; call insert_state first",
                std::any::type_name::<S>()
            )
        })
        .pending = Some(state);
}

/// Applies a pending transition for `S`: the initial entry on the first run,
/// then any requested change on later runs. On a change the current state
/// flips and a [`StateTransition`] is sent to the host's event bus, so enter
/// and exit systems reading it see each transition once. A request equal to
/// the current state is dropped. Push this onto an early stage, once per state
/// type, ahead of the systems that read the state.
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

/// The marker for a plain world system, `FnMut(&mut W)`, in a gate.
pub struct PlainMarker;

/// The marker for a system-parameter function in a gate.
pub struct ParamMarker<Marker>(PhantomData<fn() -> Marker>);

/// One gateable system: a plain world system `FnMut(&mut W)` or a
/// system-parameter function ([`crate::system_param::IntoSystem`]).
/// A single gated system and each member of a gated tuple resolve through
/// this, so a tuple may freely mix the two shapes.
pub trait GatedSystem<W, Marker> {
    /// Lowers the system to a runner the gate calls when its condition holds.
    fn into_gated_runner(self) -> impl FnMut(&mut W) + Send + 'static;
}

impl<W, F> GatedSystem<W, PlainMarker> for F
where
    F: FnMut(&mut W) + Send + 'static,
{
    fn into_gated_runner(self) -> impl FnMut(&mut W) + Send + 'static {
        self
    }
}

impl<W, Marker, F> GatedSystem<W, ParamMarker<Marker>> for F
where
    F: IntoSystem<W, Marker>,
{
    fn into_gated_runner(self) -> impl FnMut(&mut W) + Send + 'static {
        self.into_runner()
    }
}

/// The marker for a single system passed where a group is accepted.
pub struct SingleMarker<Marker>(PhantomData<fn() -> Marker>);

/// The marker for a tuple of systems passed as one gated group.
pub struct GroupMarker<Marker>(PhantomData<fn() -> Marker>);

/// One [`GatedSystem`] or a tuple of them, lowered to a single runner so a
/// gate checks its condition once and then runs the whole group in order. A
/// single system resolves through [`SingleMarker`]; a tuple through
/// [`GroupMarker`]. A tuple may mix plain world systems and system-parameter
/// functions. Implemented for tuples of up to sixteen systems.
pub trait IntoGroupRunner<W, Marker> {
    /// Builds the group's runner, initializing each member's system once.
    fn into_group_runner(self) -> impl FnMut(&mut W) + Send + 'static;
}

impl<W, Marker, S> IntoGroupRunner<W, SingleMarker<Marker>> for S
where
    S: GatedSystem<W, Marker>,
{
    fn into_group_runner(self) -> impl FnMut(&mut W) + Send + 'static {
        self.into_gated_runner()
    }
}

macro_rules! impl_group_runner_tuple {
    ($($system:ident $marker:ident $runner:ident),+) => {
        impl<W, $($system, $marker,)+> IntoGroupRunner<W, GroupMarker<($($marker,)+)>>
            for ($($system,)+)
        where
            $($system: GatedSystem<W, $marker>,)+
        {
            fn into_group_runner(self) -> impl FnMut(&mut W) + Send + 'static {
                let ($($runner,)+) = self;
                $(let mut $runner = $runner.into_gated_runner();)+
                move |world: &mut W| {
                    $($runner(world);)+
                }
            }
        }
    };
}

impl_group_runner_tuple!(S0 M0 r0);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12, S13 M13 r13
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12, S13 M13 r13, S14 M14 r14
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12, S13 M13 r13, S14 M14 r14, S15 M15 r15
);

/// Gates a system, or a tuple of systems, on an arbitrary world condition,
/// returning one runner that checks `condition` each pass and runs the group
/// only when it holds. Push the result onto a schedule.
pub fn run_if<W, Marker>(
    condition: impl Fn(&W) -> bool + Send + 'static,
    systems: impl IntoGroupRunner<W, Marker>,
) -> impl FnMut(&mut W) + Send + 'static {
    let mut runner = systems.into_group_runner();
    move |world: &mut W| {
        if condition(world) {
            runner(world);
        }
    }
}

/// Gates a system, or a tuple of systems, on the current state being `state`,
/// so one registration covers a whole group on one screen.
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

/// Gates a system, or a tuple of systems, on the current state being any of
/// `states`, so one registration covers several screens (for example both
/// `Playing` and `Paused`).
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

/// Runs a system, or a tuple of systems, once each time state `S` enters
/// `state`, reading the [`StateTransition`] event through its own cursor so
/// each entry fires exactly once. Push it after [`apply_state_transition`].
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

/// Runs a system, or a tuple of systems, once each time state `S` leaves
/// `state`, reading the [`StateTransition`] event through its own cursor so
/// each exit fires exactly once. Push it after [`apply_state_transition`].
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

/// Registers [`apply_state_transition`] for a state type onto a [`Schedule`],
/// so the transition step lands with the rest of a stage's systems.
pub trait StateScheduleExt<W> {
    /// Pushes [`apply_state_transition`] for `S` under `name`.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schedule;
    use crate::dynamic::DynWorld;
    use crate::system_param::{EventReader, ResMut, ScheduleExt};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Screen {
        Title,
        Playing,
        Paused,
    }

    struct Ticks(u32);
    struct Entered(u32);
    struct Exited(u32);

    fn state_schedule() -> Schedule<DynWorld> {
        let mut schedule = Schedule::new();
        schedule.add_state_transitions::<Screen>("screen_transitions");
        schedule
    }

    #[test]
    fn insert_and_read_state() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Title);
        assert_eq!(current_state::<Screen, _>(&world), Screen::Title);
        assert!(in_state(&world, Screen::Title));
        assert!(!in_state(&world, Screen::Playing));
    }

    #[test]
    fn transition_applies_on_next_run() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Title);
        let mut schedule = state_schedule();

        schedule.run(&mut world);
        assert_eq!(current_state::<Screen, _>(&world), Screen::Title);

        next_state(&mut world, Screen::Playing);
        assert_eq!(
            current_state::<Screen, _>(&world),
            Screen::Title,
            "the request is pending until the next transition step"
        );

        schedule.run(&mut world);
        assert_eq!(current_state::<Screen, _>(&world), Screen::Playing);
    }

    #[test]
    fn request_for_current_state_is_dropped() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Entered(0));
        let mut schedule = state_schedule();
        schedule.push("on_enter_title", on_enter(Screen::Title, count_enter));
        schedule.run(&mut world);
        assert_eq!(world.resource::<Entered>().unwrap().0, 1);

        next_state(&mut world, Screen::Title);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Entered>().unwrap().0,
            1,
            "a no-op request emits no transition, so enter does not fire again"
        );
    }

    fn tick(mut ticks: ResMut<Ticks>) {
        ticks.0 += 1;
    }
    fn count_enter(mut entered: ResMut<Entered>) {
        entered.0 += 1;
    }
    fn count_exit(mut exited: ResMut<Exited>) {
        exited.0 += 1;
    }

    #[test]
    fn while_in_gates_a_single_system() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Ticks(0));
        let mut schedule = state_schedule();
        schedule.push("tick", while_in(Screen::Playing, tick));

        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 0);

        next_state(&mut world, Screen::Playing);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 1);
    }

    #[test]
    fn while_in_gates_a_tuple_as_one_entry() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Playing);
        world.insert_resource(Ticks(0));
        world.insert_resource(Entered(0));
        let mut schedule = state_schedule();
        schedule.push(
            "group",
            while_in(Screen::Playing, (tick, count_enter, tick)),
        );

        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 2);
        assert_eq!(world.resource::<Entered>().unwrap().0, 1);
    }

    fn plain_tick(world: &mut DynWorld) {
        world.resource_mut::<Ticks>().unwrap().0 += 1;
    }

    #[test]
    fn while_in_gates_a_plain_world_system() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Playing);
        world.insert_resource(Ticks(0));
        let mut schedule = state_schedule();
        schedule.push("plain", while_in(Screen::Playing, plain_tick));

        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 1);

        next_state(&mut world, Screen::Title);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Ticks>().unwrap().0,
            1,
            "the plain world system is skipped off its state"
        );
    }

    #[test]
    fn while_in_gates_a_tuple_mixing_plain_and_param_systems() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Playing);
        world.insert_resource(Ticks(0));
        world.insert_resource(Entered(0));
        let mut schedule = state_schedule();
        schedule.push(
            "mixed",
            while_in(Screen::Playing, (plain_tick, count_enter, tick)),
        );

        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 2);
        assert_eq!(world.resource::<Entered>().unwrap().0, 1);
    }

    #[test]
    fn while_in_any_covers_several_states() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Paused);
        world.insert_resource(Ticks(0));
        let mut schedule = state_schedule();
        schedule.push("hud", while_in_any([Screen::Playing, Screen::Paused], tick));

        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 1);

        next_state(&mut world, Screen::Title);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Ticks>().unwrap().0,
            1,
            "Title is not in the gate set"
        );
    }

    #[test]
    fn run_if_gates_on_a_plain_condition() {
        let mut world = DynWorld::new();
        world.insert_resource(Ticks(0));
        world.insert_resource(Entered(0));
        let mut schedule = Schedule::new();
        schedule.push(
            "tick",
            run_if(
                |world: &DynWorld| world.resource::<Entered>().unwrap().0 == 0,
                tick,
            ),
        );

        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 1);

        world.resource_mut::<Entered>().unwrap().0 = 1;
        schedule.run(&mut world);
        assert_eq!(world.resource::<Ticks>().unwrap().0, 1);
    }

    #[test]
    fn on_enter_and_on_exit_fire_once_per_transition() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Entered(0));
        world.insert_resource(Exited(0));
        let mut schedule = state_schedule();
        schedule.push("enter_playing", on_enter(Screen::Playing, count_enter));
        schedule.push("exit_title", on_exit(Screen::Title, count_exit));

        schedule.run(&mut world);
        assert_eq!(world.resource::<Entered>().unwrap().0, 0);
        assert_eq!(world.resource::<Exited>().unwrap().0, 0);

        next_state(&mut world, Screen::Playing);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Entered>().unwrap().0, 1);
        assert_eq!(world.resource::<Exited>().unwrap().0, 1);

        schedule.run(&mut world);
        assert_eq!(world.resource::<Entered>().unwrap().0, 1);
        assert_eq!(world.resource::<Exited>().unwrap().0, 1);
    }

    #[test]
    fn initial_entry_emits_a_transition_event() {
        let mut world = DynWorld::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Seen(Vec::new()));
        let mut schedule = state_schedule();
        schedule.add_system("record", record_transitions);

        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Seen>().unwrap().0,
            vec![(None, Screen::Title)]
        );
    }

    struct Seen(Vec<(Option<Screen>, Screen)>);

    fn record_transitions(reader: EventReader<StateTransition<Screen>>, mut seen: ResMut<Seen>) {
        for transition in &reader {
            seen.0.push((transition.before, transition.after));
        }
    }
}
