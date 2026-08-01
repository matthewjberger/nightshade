use crate::commands::{move_selection, want_level, want_restart, want_screen};
use crate::data::Ask;
use crate::resources::{CoinfallResources, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;

pub fn update(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<CoinfallResources>,
    mut window: ResMut<Window>,
) {
    let handles = resources.hud;
    if handles.play_button == Entity::default() {
        return;
    }

    let mut clicked = Entity::default();
    for entity in ui_button_clicks(&frame) {
        clicked = entity;
    }
    let confirmed = actions.just_pressed(Ask::Confirm);
    let paused = actions.just_pressed(Ask::Pause);

    match resources.run.screen {
        Screen::Title => {
            if clicked == handles.play_button || confirmed {
                want_screen(&mut resources.run, Screen::Overworld);
            } else if clicked == handles.quit_button {
                window.should_exit = true;
            }
        }
        Screen::Overworld => {
            let step = i32::from(actions.just_pressed(Ask::StepRight))
                - i32::from(actions.just_pressed(Ask::StepLeft));
            if step != 0 {
                move_selection(&mut resources.run, step);
            }
            let picked = resources.dots.iter().position(|dot| *dot == clicked);
            if let Some(index) = picked
                && index < resources.run.unlocked
            {
                resources.run.selected = index;
                enter_level(&mut resources);
            } else if clicked == handles.start_button || confirmed {
                enter_level(&mut resources);
            } else if clicked == handles.overworld_title_button {
                want_screen(&mut resources.run, Screen::Title);
            }
        }
        Screen::Playing => {
            if paused {
                want_screen(&mut resources.run, Screen::Paused);
            }
        }
        Screen::Paused => {
            if clicked == handles.resume_button || paused || confirmed {
                want_screen(&mut resources.run, Screen::Playing);
            } else if clicked == handles.restart_button {
                want_restart(&mut resources.run);
                want_screen(&mut resources.run, Screen::Playing);
            } else if clicked == handles.pause_map_button {
                want_screen(&mut resources.run, Screen::Overworld);
            }
        }
        Screen::Cleared => {
            if clicked == handles.again_button || confirmed {
                want_restart(&mut resources.run);
                want_screen(&mut resources.run, Screen::Playing);
            } else if clicked == handles.cleared_map_button {
                resources.run.selected = resources.run.level;
                want_screen(&mut resources.run, Screen::Overworld);
            }
        }
    }
}

fn enter_level(resources: &mut CoinfallResources) {
    let selected = resources.run.selected;
    want_level(&mut resources.run, selected);
    want_screen(&mut resources.run, Screen::Playing);
}
