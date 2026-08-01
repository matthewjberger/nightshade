#[cfg(feature = "contract")]
pub mod data;
#[cfg(feature = "contract")]
pub mod resources;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
pub mod plugin;
#[cfg(feature = "plugin")]
pub mod prelude;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "plugin")]
pub use plugin::{StateAppExt, StatePlugin};

#[cfg(all(test, feature = "plugin"))]
mod tests {
    use crate::prelude::*;
    use crate::systems::{StateScheduleExt, on_enter, on_exit, while_in, while_in_any};
    use nightshade::ecs::system_param::{EventReader, ResMut, ScheduleExt};
    use nightshade::ecs::{Schedule, World};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Screen {
        Title,
        Playing,
        Paused,
    }

    struct Ticks(u32);
    struct Entered(u32);
    struct Exited(u32);
    struct Seen(Vec<(Option<Screen>, Screen)>);

    fn state_schedule() -> Schedule<World> {
        let mut schedule = Schedule::new();
        schedule.add_state_transitions::<Screen>("screen_transitions");
        schedule
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
    fn plain_tick(world: &mut World) {
        world.resources.get_mut::<Ticks>().unwrap().0 += 1;
    }
    fn record_transitions(reader: EventReader<StateTransition<Screen>>, mut seen: ResMut<Seen>) {
        for transition in &reader {
            seen.0.push((transition.before, transition.after));
        }
    }

    #[test]
    fn insert_and_read_state() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Title);
        assert_eq!(current_state::<Screen, _>(&world), Screen::Title);
        assert!(in_state(&world, Screen::Title));
        assert!(!in_state(&world, Screen::Playing));
    }

    #[test]
    fn transition_applies_on_next_run() {
        let mut world = World::new();
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
        let mut world = World::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Entered(0));
        let mut schedule = state_schedule();
        schedule.push("on_enter_title", on_enter(Screen::Title, count_enter));
        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Entered>().unwrap().0, 1);

        next_state(&mut world, Screen::Title);
        schedule.run(&mut world);
        assert_eq!(
            world.resources.get::<Entered>().unwrap().0,
            1,
            "a no-op request emits no transition, so enter does not fire again"
        );
    }

    #[test]
    fn while_in_gates_a_single_system() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Ticks(0));
        let mut schedule = state_schedule();
        schedule.push("tick", while_in(Screen::Playing, tick));

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Ticks>().unwrap().0, 0);

        next_state(&mut world, Screen::Playing);
        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Ticks>().unwrap().0, 1);
    }

    #[test]
    fn while_in_gates_a_tuple_as_one_entry() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Playing);
        world.insert_resource(Ticks(0));
        world.insert_resource(Entered(0));
        let mut schedule = state_schedule();
        schedule.push(
            "group",
            while_in(Screen::Playing, (tick, count_enter, tick)),
        );

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Ticks>().unwrap().0, 2);
        assert_eq!(world.resources.get::<Entered>().unwrap().0, 1);
    }

    #[test]
    fn while_in_gates_a_plain_world_system() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Playing);
        world.insert_resource(Ticks(0));
        let mut schedule = state_schedule();
        schedule.push("plain", while_in(Screen::Playing, plain_tick));

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Ticks>().unwrap().0, 1);

        next_state(&mut world, Screen::Title);
        schedule.run(&mut world);
        assert_eq!(
            world.resources.get::<Ticks>().unwrap().0,
            1,
            "the plain world system is skipped off its state"
        );
    }

    #[test]
    fn while_in_gates_a_tuple_mixing_plain_and_param_systems() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Playing);
        world.insert_resource(Ticks(0));
        world.insert_resource(Entered(0));
        let mut schedule = state_schedule();
        schedule.push(
            "mixed",
            while_in(Screen::Playing, (plain_tick, count_enter, tick)),
        );

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Ticks>().unwrap().0, 2);
        assert_eq!(world.resources.get::<Entered>().unwrap().0, 1);
    }

    #[test]
    fn while_in_any_covers_several_states() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Paused);
        world.insert_resource(Ticks(0));
        let mut schedule = state_schedule();
        schedule.push("hud", while_in_any([Screen::Playing, Screen::Paused], tick));

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Ticks>().unwrap().0, 1);

        next_state(&mut world, Screen::Title);
        schedule.run(&mut world);
        assert_eq!(
            world.resources.get::<Ticks>().unwrap().0,
            1,
            "Title is not in the gate set"
        );
    }

    #[test]
    fn on_enter_and_on_exit_fire_once_per_transition() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Entered(0));
        world.insert_resource(Exited(0));
        let mut schedule = state_schedule();
        schedule.push("enter_playing", on_enter(Screen::Playing, count_enter));
        schedule.push("exit_title", on_exit(Screen::Title, count_exit));

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Entered>().unwrap().0, 0);
        assert_eq!(world.resources.get::<Exited>().unwrap().0, 0);

        next_state(&mut world, Screen::Playing);
        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Entered>().unwrap().0, 1);
        assert_eq!(world.resources.get::<Exited>().unwrap().0, 1);

        schedule.run(&mut world);
        assert_eq!(world.resources.get::<Entered>().unwrap().0, 1);
        assert_eq!(world.resources.get::<Exited>().unwrap().0, 1);
    }

    #[test]
    fn initial_entry_emits_a_transition_event() {
        let mut world = World::new();
        insert_state(&mut world, Screen::Title);
        world.insert_resource(Seen(Vec::new()));
        let mut schedule = state_schedule();
        schedule.add_system("record", record_transitions);

        schedule.run(&mut world);
        assert_eq!(
            world.resources.get::<Seen>().unwrap().0,
            vec![(None, Screen::Title)]
        );
    }
}
