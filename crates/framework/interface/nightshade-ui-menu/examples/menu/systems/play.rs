use crate::resources::{Buttons, Screen};
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_readout::Readout;
use nightshade_state::resources::{NextState, State};
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;

pub fn take_the_clicks(
    frame: Res<RetainedUiFrame>,
    buttons: Res<Buttons>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    let clicked: Vec<Entity> = ui_button_clicks(&frame).collect();
    if clicked.is_empty() {
        return;
    }
    let wanted = match screen.current {
        Screen::Title if clicked.contains(&buttons.play) => Some(Screen::Playing),
        Screen::Title if clicked.contains(&buttons.quit) => Some(Screen::Over),
        Screen::Paused if clicked.contains(&buttons.resume) => Some(Screen::Playing),
        Screen::Paused if clicked.contains(&buttons.to_title) => Some(Screen::Title),
        Screen::Over if clicked.contains(&buttons.again) => Some(Screen::Title),
        _ => None,
    };
    if wanted.is_some() {
        next.pending = wanted;
    }
}

pub fn pause_and_play(
    input: Res<nightshade_platform::contract::input::Input>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !input
        .keyboard
        .just_pressed(nightshade_platform::prelude::KeyCode::Space)
    {
        return;
    }
    next.pending = match screen.current {
        Screen::Playing => Some(Screen::Paused),
        Screen::Paused => Some(Screen::Playing),
        _ => None,
    };
}

pub fn show_the_screen(screen: Res<State<Screen>>, mut readout: ResMut<Readout>) {
    readout.notes.clear();
    readout.notes.push(format!("screen {:?}", screen.current));
}
