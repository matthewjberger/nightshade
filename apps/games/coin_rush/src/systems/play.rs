use crate::resources::{CoinRushResources, Screen};
use crate::systems::round::{Coins, Players, begin_round, random_spot};
use crate::systems::screens::set_screen;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_countdown::queries::seconds;
use nightshade_countdown::{RanOut, Remaining, Running};
use nightshade_pickup::{PickedUp, Taken};
use nightshade_ui::prelude::ui_set_text;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_banner::Banner;

pub fn update(
    mut resources: ResMut<CoinRushResources>,
    mut remaining: ResMut<Remaining>,
    mut running: ResMut<Running>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut banner: ResMut<Banner>,
    taken: EventReader<PickedUp>,
    ran_out: EventReader<RanOut>,
    mut commands: Commands,
    mut players: Players<'_, '_>,
    mut coins: Coins<'_, '_>,
) {
    let resources = &mut *resources;
    let playing = resources.round.screen == Screen::Playing;
    players.for_each_mut(|_entity, (walking, _transform)| {
        walking.0 = if playing { 1.0 } else { 0.0 };
    });
    if !playing {
        if resources.round.screen == Screen::Title {
            banner.line.clear();
        }
        return;
    }

    for word in taken.iter() {
        resources.round.score += 1;
        let spot = random_spot(&mut resources.round.roll);
        coins.for_entity(word.pickup, |(anchor,)| anchor.0 = spot);
        commands.set(word.pickup, Taken(false));
    }

    if resources.round.score >= resources.round.goal {
        resources.round.round += 1;
        begin_round(
            resources,
            &mut remaining,
            &mut running,
            &mut commands,
            &mut coins,
        );
    }

    let round = resources.round.round;
    let score = resources.round.score;
    let goal = resources.round.goal;
    let left = seconds(&remaining);
    let handles = resources.ui;

    banner.line = format!("Round {round}   {score}/{goal}   Time {:.0}", left.ceil());

    if ran_out.iter().next().is_some() {
        if let Some(summary) = handles.over_score {
            ui_set_text(&mut commands, summary, &format!("Reached round {round}"));
        }
        set_screen(resources, &mut dirty, &mut commands, Screen::Over);
    }
}
