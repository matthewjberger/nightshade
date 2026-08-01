use crate::components::{Home, Phase, Velocity};
use crate::contract::CoinfallEvent;
use crate::queries::boxes_overlap;
use crate::resources::{CoinfallResources, Screen};
use crate::tags::Flag;
use crate::theme;
use nightshade::prelude::*;
use nightshade_pickup::{PickedUp, Taken};
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub type Coins<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Home,
        &'item Phase,
        &'item Taken,
        &'item mut LocalTransform,
    ),
>;

pub type Marks<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item Velocity, &'item LocalTransform)>;

pub type Flags<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item Home,), WithTag<Flag>>;

pub fn bob(time: Res<Time>, mut coins: Coins<'_, '_>) {
    let clock = time.uptime_seconds as f32;
    coins.for_each_mut(|_entity, (home, phase, taken, transform)| {
        if taken.0 {
            return;
        }
        let lift = (clock * theme::COIN_BOB_SPEED + phase.0).sin() * theme::COIN_BOB;
        transform.translation.y = home.0.y + lift;
    });
}

pub fn take_the_coins(
    taken: EventReader<PickedUp>,
    mut events: EventWriter<CoinfallEvent>,
    mut commands: Commands,
) {
    for word in taken.iter() {
        commands.set(word.pickup, Visibility { visible: false });
        events.send(CoinfallEvent::CoinTaken);
    }
}

pub fn reach_the_flag(
    resources: Res<CoinfallResources>,
    mut events: EventWriter<CoinfallEvent>,
    mut marks: Marks<'_, '_>,
    mut flags: Flags<'_, '_>,
) {
    if resources.run.screen != Screen::Playing {
        return;
    }
    let mut mark = None;
    marks.for_each_mut(|_entity, (_, transform)| {
        mark = Some(transform.translation);
    });
    let Some(position) = mark else {
        return;
    };
    let center = Vec2::new(position.x, position.y + theme::PLAYER_HEIGHT * 0.5);
    let half = Vec2::new(theme::PLAYER_HALF_WIDTH, theme::PLAYER_HEIGHT * 0.5);
    let flag_half = Vec2::new(theme::FLAG_REACH, theme::TILE);
    let mut reached = false;
    flags.for_each_mut(|_entity, (home,)| {
        let post = Vec2::new(home.0.x, home.0.y + theme::TILE * 0.5);
        if boxes_overlap(center, half, post, flag_half) {
            reached = true;
        }
    });
    if reached {
        events.send(CoinfallEvent::FlagReached);
    }
}
