use crate::commands::{rest, show_toast, take_shard};
use crate::components::{Facing, Used};
use crate::contract::StationKind;
use crate::data::Ask;
use crate::resources::{Party, Prompt, Rig, Screen, ScreenRequest, Toast};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Counters<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item StationKind, &'item Used, &'item LocalTransform),
>;
pub type Chests<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut Used, &'item mut Sprite)>;

pub fn find_the_prompt(
    mut prompt: ResMut<Prompt>,
    party: Res<Party>,
    walkers: Query<(&LocalTransform,), With<Facing>>,
    mut counters: Counters<'_, '_>,
) {
    let mut here = None;
    walkers.for_each(|_entity, (transform,)| {
        here = Some(Vec2::new(transform.translation.x, transform.translation.y));
    });
    let Some(here) = here else {
        prompt.station = None;
        prompt.text.clear();
        return;
    };

    let mut nearest: Option<(f32, Entity, StationKind)> = None;
    counters.for_each_mut(|entity, (station_kind, used, transform)| {
        if used.0 {
            return;
        }
        let spot = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = (spot - here).magnitude();
        if distance > theme::REACH {
            return;
        }
        if nearest.is_none_or(|(best, _, _)| distance < best) {
            nearest = Some((distance, entity, *station_kind));
        }
    });

    match nearest {
        Some((_, entity, kind)) => {
            prompt.station = Some(entity);
            prompt.kind = Some(kind);
            prompt.text = label(kind, &party);
        }
        None => {
            prompt.station = None;
            prompt.kind = None;
            prompt.text.clear();
        }
    }
}

fn label(kind: StationKind, party: &Party) -> String {
    match kind {
        StationKind::Inn => format!("E   Rest at the inn, {} gold", theme::INN_PRICE),
        StationKind::Shop => "E   Trade at the shop".to_string(),
        StationKind::Crystal => {
            if party.shards > 0 {
                "E   Feed the crystal a shard".to_string()
            } else {
                "E   Stand before the crystal".to_string()
            }
        }
        StationKind::Chest => "E   Open the chest".to_string(),
    }
}

pub fn act_on_the_prompt(
    actions: Res<Actions<Ask>>,
    prompt: Res<Prompt>,
    rig: Res<Rig>,
    mut party: ResMut<Party>,
    mut toast: ResMut<Toast>,
    mut request: ResMut<ScreenRequest>,
    mut chests: Chests<'_, '_>,
) {
    if !actions.just_pressed(Ask::Interact) {
        return;
    }
    let Some((station, kind)) = prompt.station.zip(prompt.kind) else {
        return;
    };

    match kind {
        StationKind::Inn => {
            let note = if rest(&mut party) {
                "The party sleeps. Everyone is whole again."
            } else {
                "The innkeeper shakes his head at your purse."
            };
            show_toast(&mut toast, note);
        }
        StationKind::Shop => request.wanted = Some(Screen::Shop),
        StationKind::Crystal => request.wanted = Some(Screen::Jobs),
        StationKind::Chest => {
            let note = take_shard(&mut party);
            show_toast(&mut toast, note);
            let open = rig.sheets.dungeon_tile(theme::CHEST_OPEN);
            chests.for_entity(station, |(used, sprite)| {
                used.0 = true;
                sprite.region = open;
            });
        }
    }
}
