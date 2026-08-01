use crate::components::ItemKind;
use crate::data::ask::Ask;
use crate::queries::{item_name, note_body, person};
use crate::resources::{AshfallArguments, AshfallResources, Screen};
use crate::systems::satchel;
use crate::systems::screens::{leave_run, set_screen, start_run};
use crate::tags::{LevelPiece, MenuCamera};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_platform::prelude::*;
use nightshade_render::resources::capture::ScheduledCapture;
use nightshade_scene::prelude::{LocalTransform, Name};
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::{ui_clicked, ui_set_text};
use nightshade_ui::resources::RetainedUiDirty;

pub fn update(
    actions: Res<Actions<Ask>>,
    arguments: Res<AshfallArguments>,
    capture: Res<ScheduledCapture>,
    mut resources: ResMut<AshfallResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut buttons: Query<(&UiButtonData,)>,
    mut pieces: Query<(&Name,), WithTag<LevelPiece>>,
    mut menu_cameras: Query<(&LocalTransform,), WithTag<MenuCamera>>,
    mut torches: Query<(&nightshade_flashlight::Battery,), WithTag<crate::tags::Flashlight>>,
    mut controllers: Query<(&mut CharacterControllerComponent,)>,
) {
    let resources = &mut *resources;
    let handles = resources.ui;
    let taking_a_picture = capture.path.is_some();
    let picture_is_waiting = taking_a_picture && capture.elapsed >= 2;
    let walking = resources.screen == Screen::Playing && !taking_a_picture;
    controllers.for_each_mut(|_entity, (controller,)| {
        if controller.engine_input_enabled != walking {
            controller.engine_input_enabled = walking;
            controller.velocity = Vec3::zeros();
        }
    });
    let pause_pressed = actions.just_pressed(Ask::Pause);
    let confirm_pressed = actions.just_pressed(Ask::Confirm);

    let mut level_pieces = Vec::new();
    pieces.for_each_mut(|entity, _| level_pieces.push(entity));
    let mut menu_camera = None;
    menu_cameras.for_each_mut(|entity, _| {
        if menu_camera.is_none() {
            menu_camera = Some(entity);
        }
    });

    if let Some(next) = resources.pending_screen.take() {
        match next {
            Screen::Playing => {
                start_run(
                    resources,
                    &arguments,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                );
            }
            Screen::Reading => {
                if let Some(index) = resources.run.reading {
                    if let Some(title) = handles.note_title {
                        ui_set_text(&mut commands, title, &item_name(ItemKind::Note(index)));
                    }
                    if let Some(body) = handles.note_body {
                        ui_set_text(&mut commands, body, note_body(index));
                    }
                }
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Reading,
                );
            }
            Screen::Talking => {
                say_the_line(resources, &mut commands);
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Talking,
                );
            }
            Screen::Inventory => {
                let mut charge = 0.0;
                torches.for_each_mut(|_entity, (battery,)| charge = battery.0);
                satchel::write(resources, charge, &mut commands);
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Inventory,
                );
            }
            Screen::TownMap => {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::TownMap,
                );
            }
            Screen::Dead | Screen::Escaped => {
                if next == Screen::Escaped
                    && let Some(body) = handles.escaped_body
                {
                    let kills = resources.run.kills;
                    let papers = resources.run.found_notes.iter().filter(|got| **got).count();
                    ui_set_text(
                        &mut commands,
                        body,
                        &format!(
                            "You push through the church gate and the fog thins to nothing. Behind you the town is already forgetting your name. You put {kills} of them down for good and you kept {papers} of the papers. The rest are still listening for the bell."
                        ),
                    );
                }
                leave_run(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                    menu_camera,
                    next,
                );
            }
            other => set_screen(resources, &mut window, &mut dirty, &mut commands, other),
        }
        return;
    }

    match resources.screen {
        Screen::Title => {
            if ui_clicked(&mut buttons, handles.play_button)
                || confirm_pressed
                || picture_is_waiting
            {
                start_run(
                    resources,
                    &arguments,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                );
            } else if ui_clicked(&mut buttons, handles.quit_button) {
                window.should_exit = true;
            }
        }
        Screen::Playing => {
            if pause_pressed {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Paused,
                );
            }
        }
        Screen::Paused => {
            if ui_clicked(&mut buttons, handles.resume_button) || pause_pressed {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Playing,
                );
            } else if ui_clicked(&mut buttons, handles.restart_button) {
                start_run(
                    resources,
                    &arguments,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                );
            } else if ui_clicked(&mut buttons, handles.pause_title_button) {
                leave_run(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                    menu_camera,
                    Screen::Title,
                );
            }
        }
        Screen::Reading => {
            if ui_clicked(&mut buttons, handles.note_close_button)
                || pause_pressed
                || confirm_pressed
                || actions.just_pressed(Ask::CloseNote)
            {
                resources.run.reading = None;
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Playing,
                );
            }
        }
        Screen::Talking => {
            if ui_clicked(&mut buttons, handles.talk_button)
                || confirm_pressed
                || actions.just_pressed(Ask::NextLine)
            {
                let index = resources.run.talking.unwrap_or(0);
                let last = person(index).lines.len().saturating_sub(1);
                if resources.run.said >= last {
                    resources.run.talking = None;
                    set_screen(
                        resources,
                        &mut window,
                        &mut dirty,
                        &mut commands,
                        Screen::Playing,
                    );
                } else {
                    resources.run.said += 1;
                    say_the_line(resources, &mut commands);
                }
            } else if pause_pressed {
                resources.run.talking = None;
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Playing,
                );
            }
        }
        Screen::Inventory => {
            if ui_clicked(&mut buttons, handles.inventory_close_button) {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Playing,
                );
            }
        }
        Screen::Inspect => {}
        Screen::TownMap => {
            if pause_pressed || confirm_pressed || actions.just_pressed(Ask::CloseMap) {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Playing,
                );
            }
        }
        Screen::Dead => {
            if ui_clicked(&mut buttons, handles.dead_again_button) || confirm_pressed {
                start_run(
                    resources,
                    &arguments,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                );
            } else if ui_clicked(&mut buttons, handles.dead_title_button) {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Title,
                );
            }
        }
        Screen::Escaped => {
            if ui_clicked(&mut buttons, handles.escaped_again_button) || confirm_pressed {
                start_run(
                    resources,
                    &arguments,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &level_pieces,
                );
            } else if ui_clicked(&mut buttons, handles.escaped_title_button) {
                set_screen(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    Screen::Title,
                );
            }
        }
    }
}

fn say_the_line(resources: &AshfallResources, commands: &mut Commands) {
    let Some(index) = resources.run.talking else {
        return;
    };
    let who = person(index);
    let handles = resources.ui;
    if let Some(node) = handles.talk_name {
        ui_set_text(commands, node, who.name);
    }
    if let Some(node) = handles.talk_line {
        let line = who
            .lines
            .get(resources.run.said)
            .copied()
            .unwrap_or_default();
        ui_set_text(commands, node, line);
    }
}
