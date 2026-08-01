use crate::commands::insert_state;
use crate::systems::{apply_state_transition, on_enter, on_exit};
use nightshade::World;
use nightshade::app::{App, Stage};
use nightshade::ecs::gate::IntoGroupRunner;
use nightshade::plugin::Plugin;

pub struct StatePlugin<S> {
    pub initial: S,
}

impl<S> StatePlugin<S> {
    pub fn new(initial: S) -> Self {
        Self { initial }
    }
}

impl<S: Copy + PartialEq + Send + Sync + 'static> Plugin for StatePlugin<S> {
    fn build(&self, app: &mut App) {
        insert_state(&mut app.world, self.initial);
        app.add_system(Stage::First, apply_state_transition::<S, World>);
    }
}

pub trait StateAppExt {
    fn on_enter<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        systems: impl IntoGroupRunner<World, Marker>,
    ) -> &mut Self;

    fn on_exit<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        systems: impl IntoGroupRunner<World, Marker>,
    ) -> &mut Self;
}

impl StateAppExt for App {
    fn on_enter<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        systems: impl IntoGroupRunner<World, Marker>,
    ) -> &mut Self {
        self.add_system(Stage::First, on_enter(state, systems));
        self
    }

    fn on_exit<S: Copy + PartialEq + Send + Sync + 'static, Marker>(
        &mut self,
        state: S,
        systems: impl IntoGroupRunner<World, Marker>,
    ) -> &mut Self {
        self.add_system(Stage::First, on_exit(state, systems));
        self
    }
}
