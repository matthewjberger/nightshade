use crate::commands::{
    clear_run, reset_run, spend_seconds, take_coin, take_stomp, want_restart, want_screen,
};
use crate::components::{Alive, Direction, FacingLeft, Home, Velocity};
use crate::contract::CoinfallEvent;
use crate::data::Ask;
use crate::resources::{CoinfallResources, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_pickup::Taken;
use nightshade_platform::prelude::*;
use nightshade_platformer::{Airborne, Coyote, JumpAsked};
use nightshade_scene::prelude::*;

pub type Marks<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Home,
        &'item mut Velocity,
        &'item mut Airborne,
        &'item mut Coyote,
        &'item mut JumpAsked,
        &'item mut FacingLeft,
        &'item mut LocalTransform,
    ),
>;

pub type Coins<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item Home, &'item mut Taken, &'item mut LocalTransform),
>;

pub type Walkers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Home,
        &'item mut Direction,
        &'item mut Alive,
        &'item mut LocalTransform,
    ),
>;

pub fn tick(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<CoinfallResources>,
    events: EventReader<CoinfallEvent>,
    mut marks: Marks<'_, '_>,
) {
    for event in &events {
        match event {
            CoinfallEvent::CoinTaken => take_coin(&mut resources.run),
            CoinfallEvent::WalkerStomped => take_stomp(&mut resources.run),
            CoinfallEvent::WalkerHit => spend_seconds(&mut resources.run, theme::HIT_SECONDS),
            CoinfallEvent::FlagReached => {
                clear_run(&mut resources.run);
                want_screen(&mut resources.run, Screen::Cleared);
            }
        }
    }

    if resources.run.screen != Screen::Playing {
        return;
    }

    spend_seconds(&mut resources.run, time.delta_time);

    let mut fell = false;
    marks.for_each_mut(|_entity, (_, _, _, _, _, _, transform)| {
        if transform.translation.y < theme::FALL_LIMIT {
            fell = true;
        }
    });

    if fell || resources.run.seconds <= 0.0 || actions.just_pressed(Ask::Restart) {
        want_restart(&mut resources.run);
    }
}

pub fn restart(
    mut resources: ResMut<CoinfallResources>,
    mut commands: Commands,
    mut marks: Marks<'_, '_>,
    mut coins: Coins<'_, '_>,
    mut walkers: Walkers<'_, '_>,
) {
    if !resources.run.restart_wanted {
        return;
    }
    reset_run(&mut resources.run);

    marks.for_each_mut(
        |_entity, (home, velocity, airborne, coyote, asked, facing_left, transform)| {
            transform.translation = home.0;
            velocity.0 = Vec2::zeros();
            airborne.0 = true;
            coyote.0 = 0.0;
            asked.0 = 0.0;
            facing_left.0 = false;
        },
    );

    coins.for_each_mut(|entity, (home, taken, transform)| {
        taken.0 = false;
        transform.translation = home.0;
        commands.set(entity, Visibility { visible: true });
    });

    walkers.for_each_mut(|entity, (home, direction, alive, transform)| {
        alive.0 = true;
        direction.0 = -1.0;
        transform.translation = home.0;
        commands.set(entity, Visibility { visible: true });
    });
}
