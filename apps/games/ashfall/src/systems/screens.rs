use crate::commands::{lock_cursor, stow};
use crate::components::ItemKind;
use crate::data::notes::NOTES;
use crate::resources::{
    AshfallArguments, AshfallResources, PlayerState, RunState, Screen, WorldPhase,
};
use crate::systems::level;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn refresh(resources: &AshfallResources, dirty: &mut RetainedUiDirty, commands: &mut Commands) {
    let screen = resources.screen;
    let handles = resources.ui;
    ui_set_visible(dirty, commands, handles.title, screen == Screen::Title);
    ui_set_visible(dirty, commands, handles.pause, screen == Screen::Paused);
    ui_set_visible(dirty, commands, handles.note, screen == Screen::Reading);
    ui_set_visible(dirty, commands, handles.talk, screen == Screen::Talking);
    ui_set_visible(
        dirty,
        commands,
        handles.inventory,
        screen == Screen::Inventory,
    );
    ui_set_visible(dirty, commands, handles.town_map, screen == Screen::TownMap);
    ui_set_visible(dirty, commands, handles.dead, screen == Screen::Dead);
    ui_set_visible(dirty, commands, handles.inspect, screen == Screen::Inspect);
    ui_set_visible(dirty, commands, handles.escaped, screen == Screen::Escaped);
    ui_set_visible(
        dirty,
        commands,
        handles.hud,
        screen.is_in_town() && screen != Screen::TownMap && screen != Screen::Inspect,
    );
}

pub fn set_screen(
    resources: &mut AshfallResources,
    window: &mut Window,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    screen: Screen,
) {
    resources.screen = screen;
    lock_cursor(window, screen == Screen::Playing);
    refresh(resources, dirty, commands);
}

pub fn start_run(
    resources: &mut AshfallResources,
    arguments: &AshfallArguments,
    window: &mut Window,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    pieces: &[Entity],
) {
    despawn_pieces(commands, pieces);
    let rng = resources.run.rng.clone();
    resources.player = PlayerState::default();
    resources.run = RunState {
        rng,
        found_notes: vec![false; NOTES.len()],
        phase: if arguments.otherworld {
            WorldPhase::Otherworld
        } else {
            WorldPhase::Fog
        },
        ..Default::default()
    };
    if arguments.carrying_everything {
        stow(&mut resources.satchel, ItemKind::BrassKey);
        stow(&mut resources.satchel, ItemKind::WardKey);
        stow(&mut resources.satchel, ItemKind::BoneKey);
        stow(&mut resources.satchel, ItemKind::TownMap);
    }
    level::build(commands, resources, arguments);
    set_screen(resources, window, dirty, commands, Screen::Playing);
}

pub fn leave_run(
    resources: &mut AshfallResources,
    window: &mut Window,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    pieces: &[Entity],
    menu_camera: Option<Entity>,
    screen: Screen,
) {
    despawn_pieces(commands, pieces);
    resources.player.body = None;
    resources.player.eyes = None;
    if let Some(camera) = menu_camera {
        commands.queue(move |_, resources| {
            resources.res_mut::<ActiveCamera>().0 = Some(camera);
        });
    }
    set_screen(resources, window, dirty, commands, screen);
}

fn despawn_pieces(commands: &mut Commands, pieces: &[Entity]) {
    for &piece in pieces {
        commands.despawn_recursive(piece);
    }
}
