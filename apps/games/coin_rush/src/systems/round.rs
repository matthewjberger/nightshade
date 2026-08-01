use crate::resources::{CoinRushResources, Screen};
use crate::systems::screens::set_screen;
use crate::theme;
use nightshade::prelude::*;
use nightshade_countdown::commands::start;
use nightshade_countdown::{Remaining, Running};
use nightshade_hover::Perch;
use nightshade_pickup::Taken;
use nightshade_scene::prelude::*;
use nightshade_topdown::Walking;

pub fn random_spot(roll: &mut nightshade_random::Random) -> Vec3 {
    Vec3::new(
        nightshade_random::commands::span(roll, theme::ARENA),
        theme::COIN_HEIGHT,
        nightshade_random::commands::span(roll, theme::ARENA),
    )
}

pub fn begin_round(
    resources: &mut CoinRushResources,
    remaining: &mut Remaining,
    running: &mut Running,
    commands: &mut Commands,
    coins: &mut Coins<'_, '_>,
) {
    resources.round.score = 0;
    resources.round.goal = theme::FIRST_GOAL + resources.round.round * theme::GOAL_STEP;
    start(remaining, running, theme::ROUND_SECONDS);

    let mut entities = Vec::new();
    coins.for_each_mut(|entity, _| entities.push(entity));
    for entity in entities {
        let spot = random_spot(&mut resources.round.roll);
        coins.for_entity(entity, |(perch,)| perch.0 = spot);
        commands.set(entity, Taken(false));
    }
}

pub type Coins<'world, 'item> = nightshade::ecs::system_param::Query<'world, (&'item mut Perch,)>;

pub type Players<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut Walking, &'item mut LocalTransform)>;

pub fn start_round(
    resources: &mut CoinRushResources,
    remaining: &mut Remaining,
    running: &mut Running,
    dirty: &mut nightshade_ui::resources::RetainedUiDirty,
    commands: &mut Commands,
    coins: &mut Coins<'_, '_>,
    players: &mut Players<'_, '_>,
) {
    resources.round.round = 1;
    begin_round(resources, remaining, running, commands, coins);
    players.for_each_mut(|_entity, (_, transform)| {
        transform.translation = theme::PLAYER_START;
    });
    set_screen(resources, dirty, commands, Screen::Playing);
}
