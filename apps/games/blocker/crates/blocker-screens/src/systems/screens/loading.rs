use crate::data::tips::TIPS;
use crate::resources::LoadingPanel;
use crate::theme;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_player::data::ask::Ask;
use blocker_save::resources::Save;
use blocker_screen::data::screen::Screen;
use blocker_world::resources::{Arrival, Generation};
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_state::prelude::NextState;
use nightshade_toast::commands::say;
use nightshade_toast::{Toast, ToastSettings};
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::{FlowDirection, WindowLayout};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_body_column, add_card, add_section_label, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_loading(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut loading_panel: ResMut<LoadingPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    loading_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::LOADING_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = loading_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            "Raising the world",
        );
        let body = add_body_column(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            theme::LOADING_HEIGHT - blocker_hud::theme::TITLE_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "The columns land as the workers finish them",
                blocker_hud::theme::ROW_WIDTH,
            );
            let track = tree
                .add_node()
                .size_px(blocker_hud::theme::ROW_WIDTH, theme::LOADING_BAR_HEIGHT)
                .with_rect(4.0, 1.0, blocker_hud::theme::CELL_BORDER)
                .color_raw::<UiBase>(theme::LOADING_TRACK)
                .entity();
            tree.in_parent(track, |tree| {
                loading_panel.fill = tree
                    .add_node()
                    .window(
                        Ab(vec2(theme::LOADING_BAR_INSET, theme::LOADING_BAR_INSET)),
                        Ab(vec2(
                            1.0,
                            theme::LOADING_BAR_HEIGHT - theme::LOADING_BAR_INSET * 2.0,
                        )),
                        Anchor::TopLeft,
                    )
                    .with_rect(3.0, 0.0, blocker_hud::theme::ACCENT_COLOR)
                    .color_raw::<UiBase>(blocker_hud::theme::ACCENT_COLOR)
                    .entity();
            });
            loading_panel.count = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::ROW_WIDTH,
                    blocker_hud::theme::BODY_SIZE + 10.0,
                )
                .with_text("", blocker_hud::theme::BODY_SIZE)
                .text_left()
                .color_raw::<UiBase>(blocker_hud::theme::TEXT_COLOR)
                .entity();
            loading_panel.place = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::ROW_WIDTH,
                    blocker_hud::theme::SMALL_SIZE + 10.0,
                )
                .with_text("", blocker_hud::theme::SMALL_SIZE)
                .text_left()
                .color_raw::<UiBase>(blocker_hud::theme::DIM_TEXT_COLOR)
                .entity();
            loading_panel.tip = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::ROW_WIDTH,
                    blocker_hud::theme::SMALL_SIZE + 10.0,
                )
                .with_text("", blocker_hud::theme::SMALL_SIZE)
                .text_left()
                .color_raw::<UiBase>(blocker_hud::theme::ACCENT_COLOR)
                .entity();
            loading_panel.enter_row = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::ROW_WIDTH,
                    blocker_hud::theme::ACTION_HEIGHT + 6.0,
                )
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .with_visible(false)
                .entity();
            loading_panel.enter = tree.in_parent(loading_panel.enter_row, |tree| {
                tree.add_button("Step into the world")
            });
        });
    });
    menu_show_on(&mut menus, Screen::Loading, loading_panel.panel);
    built_tree.finish();
}

pub fn watch_the_columns(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    save: Res<Save>,
    arrival: Res<Arrival>,
    generation: Res<Generation>,
    mut loading_panel: ResMut<LoadingPanel>,
    toast_settings: Res<ToastSettings>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<Screen>>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if generation.spawn.is_none() {
        return;
    }
    turn_the_tip(&time, &generation, &mut loading_panel, &mut commands);
    write_the_card(
        &arrival,
        &generation,
        &mut loading_panel,
        &mut dirty,
        &mut commands,
    );
    if !arrival.stood {
        return;
    }
    let asked = actions.just_pressed(Ask::Act)
        || actions.just_pressed(Ask::Jump)
        || ui_clicked(&mut buttons, loading_panel.enter);
    if !asked {
        return;
    }
    if !loading_panel.welcomed {
        loading_panel.welcomed = true;
        if save.found.is_none() {
            say(&mut toast, &toast_settings, "A new world");
        }
    }
    next.pending = Some(Screen::Playing);
}

fn turn_the_tip(
    time: &Time,
    generation: &Generation,
    loading_panel: &mut LoadingPanel,
    commands: &mut Commands,
) {
    loading_panel.tip_since += time.delta_time;
    if loading_panel.shown_tip.is_empty() {
        loading_panel.tip_index = generation.seed as usize % TIPS.len();
        loading_panel.tip_since = 0.0;
    } else if loading_panel.tip_since >= theme::TIP_SECONDS {
        loading_panel.tip_index = (loading_panel.tip_index + 1) % TIPS.len();
        loading_panel.tip_since = 0.0;
    }
    let tip = TIPS[loading_panel.tip_index];
    if tip != loading_panel.shown_tip {
        loading_panel.shown_tip = tip.to_string();
        ui_set_text(commands, loading_panel.tip, tip);
    }
}

fn write_the_card(
    arrival: &Arrival,
    generation: &Generation,
    loading_panel: &mut LoadingPanel,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    let count = if arrival.stood {
        "The world is ready".to_string()
    } else {
        format!("{} of {} columns", arrival.landed, arrival.wanted)
    };
    if count != loading_panel.shown_count {
        ui_set_text(commands, loading_panel.count, &count);
        loading_panel.shown_count = count;
        let share = if arrival.stood {
            1.0
        } else if arrival.wanted == 0 {
            0.0
        } else {
            arrival.landed as f32 / arrival.wanted as f32
        };
        let room = blocker_hud::theme::ROW_WIDTH - theme::LOADING_BAR_INSET * 2.0;
        ui_set_window_layout(
            dirty,
            commands,
            loading_panel.fill,
            WindowLayout {
                position: Ab(vec2(theme::LOADING_BAR_INSET, theme::LOADING_BAR_INSET)).into(),
                size: Ab(vec2(
                    (room * share).max(1.0),
                    theme::LOADING_BAR_HEIGHT - theme::LOADING_BAR_INSET * 2.0,
                ))
                .into(),
                anchor: Anchor::TopLeft,
            },
        );
    }
    if loading_panel.enter_shown != arrival.stood {
        loading_panel.enter_shown = arrival.stood;
        ui_set_visible(dirty, commands, loading_panel.enter_row, arrival.stood);
    }
    let place = format!("seed {}", generation.seed);
    if place != loading_panel.shown_place {
        ui_set_text(commands, loading_panel.place, &place);
        loading_panel.shown_place = place;
    }
}
