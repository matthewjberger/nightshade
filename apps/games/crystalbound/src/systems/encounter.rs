use crate::commands::open_battle;
use crate::contract::Place;
use crate::resources::{Battle, Field, Party, Rig, Screen, ScreenRequest};
use crate::theme;
use nightshade::prelude::*;

pub fn count_the_steps(
    mut field: ResMut<Field>,
    mut rig: ResMut<Rig>,
    mut battle: ResMut<Battle>,
    mut request: ResMut<ScreenRequest>,
    party: Res<Party>,
) {
    if field.place != Place::Dungeon || field.pending.is_some() {
        return;
    }
    if field.walked < field.next_encounter {
        return;
    }
    field.walked = 0.0;
    field.next_encounter = theme::ENCOUNTER_DISTANCE + rig.rng.f32() * theme::ENCOUNTER_SPREAD;
    open_battle(&mut battle, &mut rig, &party);
    request.wanted = Some(Screen::Battle);
}
