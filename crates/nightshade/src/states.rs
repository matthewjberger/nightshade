//! App-level states: a current-and-next holder per state type, enter and
//! exit hooks, and state-gated system registration on the [`App`] builder.
//!
//! A state type is any `Copy + PartialEq` enum. [`App::insert_state`]
//! stores the holder and registers the transition-apply step at the front
//! of [`Stage::First`], so a transition requested during a frame applies
//! at the start of the next one and no frame ever observes a mid-frame
//! flip. Enter hooks for the initial state run on the first frame.
//!
//! ```ignore
//! #[derive(Clone, Copy, PartialEq, Eq)]
//! enum Screen {
//!     Title,
//!     Playing,
//! }
//!
//! impl Plugin for MyGamePlugin {
//!     fn build(&self, app: &mut App) {
//!         app.insert_state(Screen::Title);
//!         app.on_enter(Screen::Title, build_title_ui);
//!         app.on_exit(Screen::Title, teardown_title_ui);
//!         app.add_system(Stage::Update, while_in(Screen::Playing).run(tick));
//!     }
//! }
//!
//! fn tick(world: &mut World) {
//!     if world.res::<crate::ecs::input::resources::Input>().keyboard.just_pressed(KeyCode::Space) {
//!         next_state(world, Screen::Title);
//!     }
//! }
//! ```

use crate::app::{App, Stage, game_scope};
use crate::ecs::world::World;
use freecs::dynamic::ResourceHostExt;

type StateHook = Box<dyn FnMut(&mut World) + Send + Sync>;

/// The system shapes [`App::on_enter`] and [`App::on_exit`] accept: a plain
/// `fn(&mut World)` hook and a state-and-world `fn(&mut T, &mut World)` game
/// hook, with the state lifted out of the resource map for the call so a hook
/// reads its game state directly. The `Marker` is inferred from the shape and
/// never written at call sites.
pub trait StateHookSystem<Marker>: Send + Sync + 'static {
    fn into_hook(self) -> StateHook;
}

impl<F> StateHookSystem<crate::app::WorldSystemMarker> for F
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    fn into_hook(self) -> StateHook {
        Box::new(self)
    }
}

impl<T, F> StateHookSystem<crate::app::GameSystemMarker<T>> for F
where
    T: Send + Sync + 'static,
    F: FnMut(&mut T, &mut World) + Send + Sync + 'static,
{
    fn into_hook(mut self) -> StateHook {
        Box::new(move |world: &mut World| {
            game_scope(world, |state: &mut T, world| self(state, world));
        })
    }
}

/// The holder [`App::insert_state`] stores for one state type: the
/// current state, the requested transition, and the enter and exit hooks.
pub struct AppState<S: Copy + PartialEq + Send + Sync + 'static> {
    current: S,
    next: Option<S>,
    entered: bool,
    enter: Vec<(S, StateHook)>,
    exit: Vec<(S, StateHook)>,
}

/// The current state, panicking if [`App::insert_state`] was never called
/// for `S`.
pub fn current_state<S: Copy + PartialEq + Send + Sync + 'static>(world: &World) -> S {
    world
        .ecs
        .resource::<AppState<S>>()
        .unwrap_or_else(|| {
            panic!(
                "no state of type {} was inserted, call insert_state in a plugin",
                std::any::type_name::<S>()
            )
        })
        .current
}

/// Whether the current state is `state`.
pub fn in_state<S: Copy + PartialEq + Send + Sync + 'static>(world: &World, state: S) -> bool {
    current_state::<S>(world) == state
}

/// The system shapes a gate accepts, matching every shape
/// [`SystemCollection`](crate::app::SystemCollection) accepts: a plain
/// `fn(&mut World)` system, a state-and-world `fn(&mut T, &mut World)` game
/// system, the `Res`/`ResMut` parameter shapes of both, and a tuple mixing
/// any of them. [`into_gated_runner`](GateableSystem::into_gated_runner)
/// resolves the system into one runner built once, so a gate checks its
/// condition once per pass and then runs the group.
pub trait GateableSystem<Marker> {
    fn into_gated_runner(self) -> Box<dyn FnMut(&mut World) + Send + 'static>;
}

impl<F> GateableSystem<crate::app::WorldSystemMarker> for F
where
    F: FnMut(&mut World) + Send + 'static,
{
    fn into_gated_runner(self) -> Box<dyn FnMut(&mut World) + Send + 'static> {
        Box::new(self)
    }
}

impl<T, F> GateableSystem<crate::app::GameSystemMarker<T>> for F
where
    T: Send + Sync + 'static,
    F: FnMut(&mut T, &mut World) + Send + 'static,
{
    fn into_gated_runner(mut self) -> Box<dyn FnMut(&mut World) + Send + 'static> {
        Box::new(move |world: &mut World| {
            game_scope(world, |state: &mut T, world| self(state, world));
        })
    }
}

impl<Marker, F> GateableSystem<crate::app::ParamSystemMarker<Marker>> for F
where
    F: freecs::system_param::IntoSystem<World, Marker> + Send + 'static,
{
    fn into_gated_runner(self) -> Box<dyn FnMut(&mut World) + Send + 'static> {
        Box::new(self.into_runner())
    }
}

macro_rules! impl_gateable_game_param_system {
    ($($resource:ident $binding:ident),+) => {
        impl<T, F, $($resource,)+>
            GateableSystem<crate::app::GameParamSystemMarker<T, ($($resource,)+)>> for F
        where
            T: Send + Sync + 'static,
            $($resource: freecs::system_param::ResourceParam,)+
            F: FnMut(&mut T, $($resource,)+ &mut World)
                + for<'world> FnMut(&mut T, $($resource::Item<'world>,)+ &'world mut World)
                + Send
                + 'static,
        {
            #[allow(non_snake_case)]
            fn into_gated_runner(mut self) -> Box<dyn FnMut(&mut World) + Send + 'static> {
                Box::new(move |world: &mut World| {
                    game_scope(world, |state: &mut T, world| {
                        world.resources_scope::<($($resource::Resource,)+), _>(|world, bundle| {
                            let ($($binding,)+) = bundle;
                            self(state, $($resource::build($binding),)+ world);
                        });
                    });
                })
            }
        }
    };
}

impl_gateable_game_param_system!(R0 r0);
impl_gateable_game_param_system!(R0 r0, R1 r1);
impl_gateable_game_param_system!(R0 r0, R1 r1, R2 r2);
impl_gateable_game_param_system!(R0 r0, R1 r1, R2 r2, R3 r3);

macro_rules! impl_gateable_system_for_tuple {
    ($(($marker:ident, $system_type:ident, $system_value:ident)),+) => {
        impl<$($marker),+, $($system_type: GateableSystem<$marker>),+>
            GateableSystem<($($marker,)+)> for ($($system_type,)+)
        {
            fn into_gated_runner(self) -> Box<dyn FnMut(&mut World) + Send + 'static> {
                let ($($system_value,)+) = self;
                $(let mut $system_value = $system_value.into_gated_runner();)+
                Box::new(move |world: &mut World| {
                    $($system_value(world);)+
                })
            }
        }
    };
}

impl_gateable_system_for_tuple!((M1, S1, system1));
impl_gateable_system_for_tuple!((M1, S1, system1), (M2, S2, system2));
impl_gateable_system_for_tuple!((M1, S1, system1), (M2, S2, system2), (M3, S3, system3));
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13),
    (M14, S14, system14)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13),
    (M14, S14, system14),
    (M15, S15, system15)
);
impl_gateable_system_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13),
    (M14, S14, system14),
    (M15, S15, system15),
    (M16, S16, system16)
);

/// Gates a system on an arbitrary world condition, returning a system for
/// [`App::add_system`]: the wrapped system runs only on the passes where
/// `condition` holds. Takes either system shape, or a tuple of them, so one
/// condition gates a whole group in a single schedule entry.
///
/// ```ignore
/// app.add_systems(Stage::Update, run_if(is_running, (physics, ai, hud)));
/// ```
pub fn run_if<Marker>(
    condition: impl Fn(&World) -> bool + Send + 'static,
    system: impl GateableSystem<Marker>,
) -> impl FnMut(&mut World) + Send + 'static {
    let mut runner = system.into_gated_runner();
    move |world: &mut World| {
        if condition(world) {
            runner(world);
        }
    }
}

/// A composable state gate built by [`while_in`], [`while_in_any`], or
/// [`while_not_in`]. Included states are OR'd (the gate opens when the current
/// state matches any of them, or any state when none are listed); excluded
/// states are subtracted (the gate stays shut while the current state matches
/// any of them). Finish with [`run`](StateGate::run), which wraps a system,
/// tuple of systems included, into one gated schedule entry.
///
/// ```ignore
/// app.add_systems(Stage::Update, while_in(Screen::Playing).run((tick, animate)));
/// app.add_systems(Stage::Update, while_in_any([Screen::Playing, Screen::Paused]).run(hud));
/// app.add_systems(Stage::Update, while_not_in([Screen::Loading]).and_not([Screen::Title]).run(audio));
/// ```
pub struct StateGate<S> {
    include: Vec<S>,
    exclude: Vec<S>,
}

impl<S: Copy + PartialEq + Send + Sync + 'static> StateGate<S> {
    /// Adds states to the included set, OR'd with any already listed.
    pub fn or(mut self, states: impl IntoIterator<Item = S>) -> Self {
        self.include.extend(states);
        self
    }

    /// Subtracts states: the gate stays shut whenever the current state
    /// matches any of them, even if it also matches an included state.
    pub fn and_not(mut self, states: impl IntoIterator<Item = S>) -> Self {
        self.exclude.extend(states);
        self
    }

    /// Wraps `system` (either system shape, or a tuple of them) into a single
    /// gated schedule entry for [`App::add_system`], checking the gate once per
    /// pass before running the group.
    pub fn run<Marker>(
        self,
        system: impl GateableSystem<Marker>,
    ) -> impl FnMut(&mut World) + Send + 'static {
        let StateGate { include, exclude } = self;
        run_if(
            move |world| {
                let current = current_state::<S>(world);
                (include.is_empty() || include.contains(&current)) && !exclude.contains(&current)
            },
            system,
        )
    }
}

/// Starts a [`StateGate`] that opens while the current state is `state`.
pub fn while_in<S: Copy + PartialEq + Send + Sync + 'static>(state: S) -> StateGate<S> {
    StateGate {
        include: vec![state],
        exclude: Vec::new(),
    }
}

/// Starts a [`StateGate`] that opens while the current state is any of
/// `states`, so one registration covers several screens (for example running a
/// system in both `Playing` and `Paused`).
pub fn while_in_any<S: Copy + PartialEq + Send + Sync + 'static>(
    states: impl IntoIterator<Item = S>,
) -> StateGate<S> {
    StateGate {
        include: states.into_iter().collect(),
        exclude: Vec::new(),
    }
}

/// Starts a [`StateGate`] that opens in every state except `states`, so a
/// system runs everywhere but the listed screens.
pub fn while_not_in<S: Copy + PartialEq + Send + Sync + 'static>(
    states: impl IntoIterator<Item = S>,
) -> StateGate<S> {
    StateGate {
        include: Vec::new(),
        exclude: states.into_iter().collect(),
    }
}

/// Requests a transition, applied at the start of the next frame: exit
/// hooks for the old state run, the state flips, enter hooks for the new
/// state run. A request for the current state is dropped.
pub fn next_state<S: Copy + PartialEq + Send + Sync + 'static>(world: &mut World, state: S) {
    world
        .ecs
        .resource_mut::<AppState<S>>()
        .unwrap_or_else(|| {
            panic!(
                "no state of type {} was inserted, call insert_state in a plugin",
                std::any::type_name::<S>()
            )
        })
        .next = Some(state);
}

fn apply_state_transitions<S: Copy + PartialEq + Send + Sync + 'static>(world: &mut World) {
    loop {
        let Some(state) = world.ecs.resource_mut::<AppState<S>>() else {
            return;
        };
        let (previous, next) = if !state.entered {
            state.entered = true;
            (None, state.current)
        } else {
            let Some(next) = state.next.take() else {
                return;
            };
            if next == state.current {
                continue;
            }
            let previous = state.current;
            state.current = next;
            (Some(previous), next)
        };
        let mut exit = std::mem::take(&mut state.exit);
        let mut enter = std::mem::take(&mut state.enter);
        if let Some(previous) = previous {
            for (target, hook) in &mut exit {
                if *target == previous {
                    hook(world);
                }
            }
        }
        for (target, hook) in &mut enter {
            if *target == next {
                hook(world);
            }
        }
        let Some(state) = world.ecs.resource_mut::<AppState<S>>() else {
            return;
        };
        state.exit = exit;
        state.enter = enter;
    }
}

impl App {
    /// Inserts the state holder for `S` starting at `initial` and
    /// registers the transition-apply step onto [`Stage::First`]. Enter
    /// hooks for `initial` run on the first frame. Panics if a state of
    /// type `S` was already inserted.
    pub fn insert_state<S: Copy + PartialEq + Send + Sync + 'static>(
        &mut self,
        initial: S,
    ) -> &mut Self {
        if self.world.ecs.resource::<AppState<S>>().is_some() {
            panic!(
                "a state of type {} was already inserted",
                std::any::type_name::<S>()
            );
        }
        self.world.ecs.insert_resource(AppState {
            current: initial,
            next: None,
            entered: false,
            enter: Vec::new(),
            exit: Vec::new(),
        });
        self.add_system(Stage::First, apply_state_transitions::<S>);
        self
    }

    /// Runs `system` when a transition lands on `state`, after the old
    /// state's exit hooks, with full world access. Enter hooks for the
    /// initial state run on the first frame. Requires
    /// [`insert_state`](App::insert_state) first.
    pub fn on_enter<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        system: impl StateHookSystem<Marker>,
    ) -> &mut Self {
        let hook = system.into_hook();
        self.state_holder::<S>().enter.push((state, hook));
        self
    }

    /// Runs `system` when a transition leaves `state`, before the new
    /// state's enter hooks. Takes either hook shape. Requires
    /// [`insert_state`](App::insert_state) first.
    pub fn on_exit<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        system: impl StateHookSystem<Marker>,
    ) -> &mut Self {
        let hook = system.into_hook();
        self.state_holder::<S>().exit.push((state, hook));
        self
    }

    fn state_holder<S: Copy + PartialEq + Send + Sync + 'static>(&mut self) -> &mut AppState<S> {
        self.world
            .ecs
            .resource_mut::<AppState<S>>()
            .unwrap_or_else(|| {
                panic!(
                    "no state of type {} was inserted, call insert_state first",
                    std::any::type_name::<S>()
                )
            })
    }
}
