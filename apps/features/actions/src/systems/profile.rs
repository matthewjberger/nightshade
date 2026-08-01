use crate::data::Ask;
use nightshade::prelude::*;
use nightshade_input::prelude::{Actions, Bindings, profiles, wear, worn};

pub fn swap_the_profile(actions: Res<Actions<Ask>>, mut bindings: ResMut<Bindings<Ask>>) {
    if !actions.just_pressed(Ask::NextProfile) {
        return;
    }
    let names: Vec<String> = profiles(&bindings)
        .into_iter()
        .map(str::to_string)
        .collect();
    let Some(here) = names.iter().position(|name| name == worn(&bindings)) else {
        return;
    };
    let next = names[(here + 1) % names.len()].clone();
    wear(&mut bindings, &next);
}
