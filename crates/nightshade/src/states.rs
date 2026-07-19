//! App-builder sugar over nightshade_ecs's state machine (the `state` feature). The
//! machinery lives in [`nightshade_ecs::state`]; this wires it onto the [`App`] builder
//! and the [`Stage::First`] transition step, and adapts `current_state` to the
//! engine [`World`] so call sites name only the state type.
//!
//! ```ignore
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
//!     if world.res::<crate::ecs::input::resources::Input>().keyboard.just_pressed(KeyCode::Space) {
//!         next_state(world, Screen::Title);
//!     }
//! }
//! ```

use crate::app::{App, Stage};
use crate::ecs::world::World;

/// The current value of state type `S`, adapting
/// [`nightshade_ecs::state::current_state`] to the engine [`World`] so call sites name
/// only `S`.
pub fn current_state<S: Copy + PartialEq + Send + Sync + 'static>(world: &World) -> S {
    nightshade_ecs::state::current_state::<S, World>(world)
}

impl App {
    /// Inserts the [`State`](nightshade_ecs::state::State) and
    /// [`NextState`](nightshade_ecs::state::NextState) resources for `S` at `initial`
    /// and registers the transition-apply step onto [`Stage::First`], so a
    /// transition requested during a frame lands at the start of the next and
    /// no frame observes a mid-frame flip. Enter hooks for `initial` run on the
    /// first frame. Panics if a state of type `S` was already inserted.
    pub fn insert_state<S: Copy + PartialEq + Send + Sync + 'static>(
        &mut self,
        initial: S,
    ) -> &mut Self {
        nightshade_ecs::state::insert_state(&mut self.world, initial);
        self.add_system(
            Stage::First,
            nightshade_ecs::state::apply_state_transition::<S, World>,
        );
        self
    }

    /// Runs `systems` once each time `S` enters `state`, after the transition
    /// step, reading the emitted
    /// [`StateTransition`](nightshade_ecs::state::StateTransition) through its own
    /// cursor so each entry fires exactly once. Takes a single system or a
    /// tuple. Requires [`insert_state`](App::insert_state) first.
    pub fn on_enter<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        systems: impl nightshade_ecs::state::IntoGroupRunner<World, Marker>,
    ) -> &mut Self {
        self.add_system(
            Stage::First,
            nightshade_ecs::state::on_enter(state, systems),
        );
        self
    }

    /// Runs `systems` once each time `S` leaves `state`, after the transition
    /// step. Takes a single system or a tuple. Requires
    /// [`insert_state`](App::insert_state) first.
    pub fn on_exit<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        systems: impl nightshade_ecs::state::IntoGroupRunner<World, Marker>,
    ) -> &mut Self {
        self.add_system(Stage::First, nightshade_ecs::state::on_exit(state, systems));
        self
    }
}
