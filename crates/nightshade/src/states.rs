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
//!         app.add_system(Stage::Update, while_in(Screen::Playing, tick));
//!     }
//! }
//!
//! fn tick(world: &mut World) {
//!     if world.resources.input.keyboard.just_pressed(KeyCode::Space) {
//!         next_state(world, Screen::Title);
//!     }
//! }
//! ```

use crate::app::{App, Stage, game_scope};
use crate::ecs::world::World;

type StateHook = Box<dyn FnMut(&mut World) + Send + Sync>;

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

/// The system shapes [`while_in`] gates: plain world systems and
/// state-and-world game systems, sharing the marker types of
/// [`SystemCollection`](crate::app::SystemCollection).
pub trait GateableSystem<Marker>: Send + 'static {
    fn run(&mut self, world: &mut World);
}

impl<F> GateableSystem<crate::app::WorldSystemMarker> for F
where
    F: FnMut(&mut World) + Send + 'static,
{
    fn run(&mut self, world: &mut World) {
        self(world);
    }
}

impl<T, F> GateableSystem<crate::app::GameSystemMarker<T>> for F
where
    T: Send + Sync + 'static,
    F: FnMut(&mut T, &mut World) + Send + 'static,
{
    fn run(&mut self, world: &mut World) {
        game_scope(world, |state: &mut T, world| self(state, world));
    }
}

/// Gates a system on the current state, returning a system for
/// [`App::add_system`]: the wrapped system runs only while the state is
/// `state`. Takes either system shape.
///
/// ```ignore
/// app.add_system(Stage::Update, while_in(Screen::Playing, tick));
/// ```
pub fn while_in<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
    state: S,
    mut system: impl GateableSystem<Marker>,
) -> impl FnMut(&mut World) + Send + 'static {
    move |world: &mut World| {
        if in_state(world, state) {
            system.run(world);
        }
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
    pub fn on_enter<S: Copy + PartialEq + Send + Sync + 'static>(
        &mut self,
        state: S,
        system: impl FnMut(&mut World) + Send + Sync + 'static,
    ) -> &mut Self {
        self.state_holder::<S>()
            .enter
            .push((state, Box::new(system)));
        self
    }

    /// Runs `system` when a transition leaves `state`, before the new
    /// state's enter hooks. Requires [`insert_state`](App::insert_state)
    /// first.
    pub fn on_exit<S: Copy + PartialEq + Send + Sync + 'static>(
        &mut self,
        state: S,
        system: impl FnMut(&mut World) + Send + Sync + 'static,
    ) -> &mut Self {
        self.state_holder::<S>()
            .exit
            .push((state, Box::new(system)));
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
