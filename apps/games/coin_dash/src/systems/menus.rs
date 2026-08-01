use crate::data::{Ask, Avatar, MenuCamera};
use crate::resources::{CoinDashResources, Screen};
use crate::systems::screens::{leave_run, set_cursor_locked, set_screen, start_game};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::ui_clicked;
use nightshade_ui::resources::RetainedUiDirty;

pub fn update(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<CoinDashResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut buttons: Query<(&UiButtonData,)>,
    mut avatar_query: Query<(&LocalTransform,), WithTag<Avatar>>,
    mut camera_query: Query<(&LocalTransform,), WithTag<MenuCamera>>,
) {
    let resources = &mut *resources;
    let handles = resources.ui;
    let pause_pressed = actions.just_pressed(Ask::Pause);

    let mut avatars = Vec::new();
    avatar_query.for_each_mut(|entity, _| avatars.push(entity));
    let mut menu_camera = None;
    camera_query.for_each_mut(|entity, _| {
        if menu_camera.is_none() {
            menu_camera = Some(entity);
        }
    });

    match resources.round.screen {
        Screen::Title => {
            if ui_clicked(&mut buttons, handles.play_button) {
                start_game(resources, &mut window, &mut dirty, &mut commands, &avatars);
            } else if ui_clicked(&mut buttons, handles.quit_button) {
                window.should_exit = true;
            }
        }
        Screen::Playing => {
            if pause_pressed {
                set_cursor_locked(&mut window, false);
                set_screen(resources, &mut dirty, &mut commands, Screen::Paused);
            }
        }
        Screen::Paused => {
            if ui_clicked(&mut buttons, handles.resume_button) || pause_pressed {
                set_cursor_locked(&mut window, true);
                set_screen(resources, &mut dirty, &mut commands, Screen::Playing);
            } else if ui_clicked(&mut buttons, handles.restart_button) {
                start_game(resources, &mut window, &mut dirty, &mut commands, &avatars);
            } else if ui_clicked(&mut buttons, handles.pause_title_button) {
                leave_run(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &avatars,
                    menu_camera,
                    Screen::Title,
                );
            }
        }
        Screen::Over => {
            if ui_clicked(&mut buttons, handles.again_button) {
                start_game(resources, &mut window, &mut dirty, &mut commands, &avatars);
            } else if ui_clicked(&mut buttons, handles.over_title_button) {
                leave_run(
                    resources,
                    &mut window,
                    &mut dirty,
                    &mut commands,
                    &avatars,
                    menu_camera,
                    Screen::Title,
                );
            }
        }
    }
}
