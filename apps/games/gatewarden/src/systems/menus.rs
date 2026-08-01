use crate::commands::run;
use crate::commands::screens::set_screen;
use crate::components::{Generation, OfferIndex};
use crate::contract::Screen;
use crate::data::ask::{Ask, LANES};
use crate::data::climb::{gate_at, step_at};
use crate::queries::run::scene_text;
use crate::resources::{GatewardenResources, Mode};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::ui_clicked;
use nightshade_ui::resources::RetainedUiDirty;

pub fn update(
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    mut resources: ResMut<GatewardenResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut buttons: Query<(&UiButtonData,)>,
    mut offers: Query<(&OfferIndex, &Generation)>,
) {
    let resources = &mut *resources;
    let handles = resources.ui;
    if actions.just_pressed(Ask::Pause) && resources.screen != Screen::Title {
        resources.paused = !resources.paused;
    }
    if resources.paused {
        if ui_clicked(&mut buttons, handles.pause_resume_button) {
            resources.paused = false;
        } else if ui_clicked(&mut buttons, handles.pause_menu_button) {
            resources.paused = false;
            set_screen(resources, &mut dirty, &mut commands, Screen::Title);
        } else if ui_clicked(&mut buttons, handles.pause_quit_button) {
            window.should_exit = true;
        }
        return;
    }
    let confirm = actions.just_pressed(Ask::Confirm);
    let alternate = actions.just_pressed(Ask::Alternate);

    match resources.screen {
        Screen::Title => {
            if ui_clicked(&mut buttons, handles.new_run_button) || confirm {
                run::start_run(resources, entropy(&time));
                set_screen(resources, &mut dirty, &mut commands, Screen::Scene);
            } else if ui_clicked(&mut buttons, handles.story_button) {
                run::start_story(resources);
                set_screen(resources, &mut dirty, &mut commands, Screen::Scene);
            } else if ui_clicked(&mut buttons, handles.learn_button) {
                run::start_tutorial(resources);
                set_screen(resources, &mut dirty, &mut commands, Screen::Scene);
            } else if ui_clicked(&mut buttons, handles.quit_button) {
                window.should_exit = true;
            }
        }
        Screen::Map => {
            let column = resources.run.column;
            if let Some(gate) = gate_at(column) {
                let node = handles.map_gate_nodes[gate as usize - 1];
                if ui_clicked(&mut buttons, handles.map_climb_button)
                    || ui_clicked(&mut buttons, node)
                    || confirm
                {
                    set_screen(resources, &mut dirty, &mut commands, Screen::Scene);
                }
            } else if let Some(step) = step_at(column) {
                let mut chosen = None;
                for (lane, ask) in LANES.iter().enumerate() {
                    let keyed = actions.just_pressed(*ask);
                    if ui_clicked(&mut buttons, handles.map_stop_nodes[step][lane]) || keyed {
                        chosen = Some(lane);
                    }
                }
                if let Some(lane) = chosen {
                    let screen = run::take_stop(resources, lane);
                    set_screen(resources, &mut dirty, &mut commands, screen);
                }
            }
        }
        Screen::Scene => {
            let scene = scene_text(&resources.run);
            let next = match scene.fork {
                Some(fork) => {
                    if ui_clicked(&mut buttons, handles.scene_first_button) || alternate {
                        Some(run::choose(resources, fork.first.outcome))
                    } else if ui_clicked(&mut buttons, handles.scene_second_button) || confirm {
                        Some(run::choose(resources, fork.second.outcome))
                    } else {
                        None
                    }
                }
                None => {
                    if ui_clicked(&mut buttons, handles.scene_continue_button) || confirm {
                        Some(run::continue_scene(resources))
                    } else {
                        None
                    }
                }
            };
            if let Some(screen) = next
                && screen != Screen::Scene
            {
                set_screen(resources, &mut dirty, &mut commands, screen);
            }
        }
        Screen::Battle => {}
        Screen::Shop => {
            for (index, button) in handles.offer_buttons.iter().enumerate() {
                if ui_clicked(&mut buttons, *button) {
                    run::buy(&mut resources.run, index);
                }
            }
            let mut offer_nodes = Vec::new();
            offers.for_each_mut(|entity, (index, _)| offer_nodes.push((entity, index.0)));
            let picked: Vec<usize> = offer_nodes
                .into_iter()
                .filter(|(entity, _)| ui_clicked(&mut buttons, *entity))
                .map(|(_, index)| index)
                .collect();
            for index in picked {
                run::buy(&mut resources.run, index);
            }
            if ui_clicked(&mut buttons, handles.reroll_button) || alternate {
                run::reroll(&mut resources.run);
            } else if ui_clicked(&mut buttons, handles.strike_button) {
                run::strike(&mut resources.run);
            } else if ui_clicked(&mut buttons, handles.next_button) || confirm {
                let screen = run::leave_shop(&mut resources.run);
                set_screen(resources, &mut dirty, &mut commands, screen);
            }
        }
        Screen::Over => {
            if ui_clicked(&mut buttons, handles.over_new_run_button) || confirm {
                if resources.run.mode == Mode::Story {
                    run::restart_chapter(resources);
                } else {
                    run::start_run(resources, entropy(&time));
                }
                set_screen(resources, &mut dirty, &mut commands, Screen::Scene);
            } else if ui_clicked(&mut buttons, handles.over_title_button) {
                set_screen(resources, &mut dirty, &mut commands, Screen::Title);
            }
        }
    }
}

fn entropy(time: &Time) -> u64 {
    (u64::from(time.delta_time.to_bits()) << 21) ^ u64::from(time.frames_per_second.to_bits())
}
