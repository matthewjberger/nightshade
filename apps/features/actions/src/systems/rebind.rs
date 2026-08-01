use crate::data::Ask;
use crate::resources::ActionsResources;
use nightshade::prelude::*;
use nightshade_gamepad::contract::Gamepad;
use nightshade_input::prelude::{Actions, Binding, Bindings, bind_only, worn};
use nightshade_platform::prelude::{Input, KeyCode};

pub fn rebind_the_jump(
    actions: Res<Actions<Ask>>,
    input: Res<Input>,
    pad: Option<Res<Gamepad>>,
    mut bindings: ResMut<Bindings<Ask>>,
    mut resources: ResMut<ActionsResources>,
) {
    if !resources.listening {
        if actions.just_pressed(Ask::Rebind) {
            resources.listening = true;
        }
        return;
    }
    let mut keys: Vec<KeyCode> = input.keyboard.just_pressed_keys.iter().copied().collect();
    keys.sort_by_key(|key| format!("{key:?}"));
    let picked = keys.first().map(|key| Binding::Key(*key)).or_else(|| {
        let pad = pad.as_deref().filter(|pad| pad.connected)?;
        let mut buttons: Vec<_> = pad.just_pressed.iter().copied().collect();
        buttons.sort_by_key(|button| format!("{button:?}"));
        buttons.first().map(|button| Binding::Pad(*button))
    });
    let Some(picked) = picked else {
        return;
    };
    resources.listening = false;
    if picked == Binding::Key(KeyCode::Escape) {
        return;
    }
    let profile = worn(&bindings).to_string();
    bind_only(&mut bindings, &profile, Ask::Jump, picked);
}
