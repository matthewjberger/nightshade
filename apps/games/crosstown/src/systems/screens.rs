use crate::commands::interface::say;
use crate::components::{Combo, Health, Holding, Piece};
use crate::data;
use crate::data::Ask;
use crate::resources::{Interface, Mode, Ring, Screen, Talk};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::resources::NextState;
use nightshade_ui::prelude::{ui_button_clicks, ui_set_text, ui_set_visible};
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn open_title(
    handles: Res<Interface>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, handles.title, true);
    show(&mut dirty, &mut commands, handles.pause, false);
    show(&mut dirty, &mut commands, handles.over, false);
    show(&mut dirty, &mut commands, handles.head_up, false);
}

pub fn open_play(
    handles: Res<Interface>,
    mut talk: ResMut<Talk>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, handles.title, false);
    show(&mut dirty, &mut commands, handles.pause, false);
    show(&mut dirty, &mut commands, handles.over, false);
    show(&mut dirty, &mut commands, handles.head_up, true);
    if talk.opened {
        return;
    }
    talk.opened = true;
    for line in data::OPENING {
        say(&mut talk, line);
    }
}

pub fn open_pause(
    handles: Res<Interface>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, handles.pause, true);
}

pub fn open_over(
    ring: Res<Ring>,
    handles: Res<Interface>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, handles.over, true);
    let title = if ring.won {
        "THE BLOCK IS YOURS"
    } else {
        "GAME OVER"
    };
    ui_set_text(&mut commands, handles.over_title, title);
    let tail = if ring.mode == Mode::Arcade {
        format!("ROUND {}", ring.round + 1)
    } else {
        format!("STAGE {}", ring.stage + 1)
    };
    ui_set_text(
        &mut commands,
        handles.over_score,
        &format!("SCORE {}     {tail}", ring.score),
    );
}

pub fn choose_title(
    handles: Res<Interface>,
    frame: Res<RetainedUiFrame>,
    mut ring: ResMut<Ring>,
    mut window: ResMut<Window>,
    mut next: ResMut<NextState<Screen>>,
) {
    for entity in ui_button_clicks(&frame) {
        if entity == handles.story {
            start(&mut ring, Mode::Story);
            next.pending = Some(Screen::Play);
        } else if entity == handles.arcade {
            start(&mut ring, Mode::Arcade);
            next.pending = Some(Screen::Play);
        } else if entity == handles.leave {
            window.should_exit = true;
        }
    }
}

pub fn choose_pause(
    handles: Res<Interface>,
    frame: Res<RetainedUiFrame>,
    actions: Res<Actions<Ask>>,
    mut next: ResMut<NextState<Screen>>,
) {
    for entity in ui_button_clicks(&frame) {
        if entity == handles.resume {
            next.pending = Some(Screen::Play);
        } else if entity == handles.menu {
            next.pending = Some(Screen::Title);
        }
    }
    if actions.just_pressed(Ask::Pause) {
        next.pending = Some(Screen::Play);
    }
}

pub fn choose_over(
    handles: Res<Interface>,
    frame: Res<RetainedUiFrame>,
    mut ring: ResMut<Ring>,
    mut next: ResMut<NextState<Screen>>,
) {
    for entity in ui_button_clicks(&frame) {
        if entity == handles.again {
            let mode = ring.mode;
            start(&mut ring, mode);
            next.pending = Some(Screen::Play);
        } else if entity == handles.back {
            next.pending = Some(Screen::Title);
        }
    }
}

pub fn hold(actions: Res<Actions<Ask>>, mut next: ResMut<NextState<Screen>>) {
    if actions.just_pressed(Ask::Pause) {
        next.pending = Some(Screen::Pause);
    }
}

pub fn reset(
    mut ring: ResMut<Ring>,
    mut commands: Commands,
    pieces: Query<(&Piece,)>,
    heroes: Query<(&mut Health, &mut Holding, &Combo)>,
) {
    if !ring.restart {
        return;
    }
    ring.restart = false;
    pieces.for_each(|entity, (piece,)| {
        if piece.0 != u32::MAX {
            commands.despawn_recursive(entity);
        }
    });
    heroes.for_each(|_entity, (health, holding, _)| {
        health.0 = data::HERO_HEALTH;
        holding.0 = 0;
    });
    ring.hero_health = data::HERO_HEALTH;
}

fn start(ring: &mut Ring, mode: Mode) {
    ring.mode = mode;
    ring.stage = 0;
    ring.round = 0;
    ring.score = 0;
    ring.dead = false;
    ring.won = false;
    ring.built = false;
    ring.opened = false;
    ring.last_wave = false;
    ring.wave = 0;
    ring.restart = true;
    ring.gate.clear();
    ring.piece += 1;
}

fn show(dirty: &mut RetainedUiDirty, commands: &mut Commands, entity: Entity, visible: bool) {
    if entity == Entity::default() {
        return;
    }
    ui_set_visible(dirty, commands, entity, visible);
}
