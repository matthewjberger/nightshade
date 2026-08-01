use crate::resources::Screens;
use crate::resources::{SettingsPanel, Tuning};
use crate::theme;
use blocker_blocks::resources::VoxelSettings;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_hud::data::settings::Settings;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_player::commands::set_watching;
use blocker_player::components::{Motion, Velocity, WatchSpeed};
use blocker_save::resources::Save;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui_config::prelude::build_config_rows;
use nightshade_ui_menu::prelude::{
    CardStyle, add_action_button, add_body_column, add_card, add_cell_text, add_footer_buttons,
    add_row_strip, add_scrolling_body, add_section_label, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_settings(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut settings_panel: ResMut<SettingsPanel>,
    tuning: Tuning,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    settings_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::SETTINGS_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = settings_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "Settings");
        let body_height = theme::SETTINGS_HEIGHT
            - blocker_hud::theme::TITLE_HEIGHT
            - blocker_hud::theme::FOOTER_HEIGHT;
        let padded = add_body_column(tree, &style, blocker_hud::theme::MENU_WIDTH, body_height);
        let body = tree.in_parent(padded, |tree| {
            add_scrolling_body(tree, body_height - 2.0 * style.body_padding)
        });
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "How you move through the world",
                blocker_hud::theme::ROW_WIDTH,
            );
            let strip = add_row_strip(tree, &style, 2, blocker_hud::theme::ROW_WIDTH);
            tree.in_parent(strip, |tree| {
                add_cell_text(
                    tree,
                    &style,
                    "Watch the world and pass through it",
                    theme::SETTING_NAME_WIDTH,
                    blocker_hud::theme::BODY_SIZE,
                    blocker_hud::theme::TEXT_COLOR,
                );
                settings_panel.watch_toggle = add_action_button(tree, &style, "Off");
            });
            add_section_label(
                tree,
                &style,
                "Your world on disk",
                blocker_hud::theme::ROW_WIDTH,
            );
            let strip = add_row_strip(tree, &style, 3, blocker_hud::theme::ROW_WIDTH);
            tree.in_parent(strip, |tree| {
                add_cell_text(
                    tree,
                    &style,
                    "The world saves itself while you play",
                    theme::SETTING_NAME_WIDTH,
                    blocker_hud::theme::BODY_SIZE,
                    blocker_hud::theme::TEXT_COLOR,
                );
                settings_panel.save = add_action_button(tree, &style, "Save now");
            });
            *tuning.interface_rows = build_config_rows(tree, tuning.interface);
            *tuning.weather_rows = build_config_rows(tree, tuning.weather);
            *tuning.player_rows = build_config_rows(tree, tuning.player);
            *tuning.crew_rows = build_config_rows(tree, tuning.crew);
            *tuning.beast_rows = build_config_rows(tree, tuning.beasts);
        });
        let buttons = add_footer_buttons(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            &["Back", "Save and go to the menu"],
        );
        settings_panel.close = buttons[0];
        settings_panel.title = buttons[1];
    });
    menu_show_on(&mut menus, Screen::Settings, settings_panel.panel);
    built_tree.finish();
}

pub fn settings_buttons(
    settings_panel: Res<SettingsPanel>,
    hero: Res<Hero>,
    screens: Res<Screens>,
    mut save: ResMut<Save>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
    mut bodies: Query<(&mut Motion, &mut Velocity, &mut WatchSpeed)>,
) {
    if ui_clicked(&mut buttons, settings_panel.close) {
        next.pending = Some(screens.return_to);
        return;
    }
    if ui_clicked(&mut buttons, settings_panel.watch_toggle)
        && let Some(body) = hero.0
    {
        bodies.for_entity(body, |(motion, velocity, watch)| {
            let watching = *motion != Motion::Watch;
            set_watching(motion, velocity, watch, watching);
        });
    }
    if ui_clicked(&mut buttons, settings_panel.save) {
        save.autosave.ask();
    }
    if ui_clicked(&mut buttons, settings_panel.title) {
        save.autosave.ask();
        next.pending = Some(Screen::Title);
    }
}

pub fn show_the_toggles(
    hero: Res<Hero>,
    mut settings_panel: ResMut<SettingsPanel>,
    mut commands: Commands,
    mut bodies: Query<(&Motion,)>,
) {
    if settings_panel.watch_toggle == Entity::default() {
        return;
    }
    let Some(watching) = read_the_hero(&hero, &mut bodies, |(motion,)| *motion == Motion::Watch)
    else {
        return;
    };
    if settings_panel.shown_watching != Some(watching) {
        settings_panel.shown_watching = Some(watching);
        ui_set_text(
            &mut commands,
            settings_panel.watch_toggle,
            if watching { "On" } else { "Off" },
        );
    }
}

pub fn keep_the_sight(config: Res<Config<Settings>>, mut blocks: ResMut<VoxelSettings>) {
    let step = config
        .choice(Settings::Sight)
        .min(theme::SIGHT_RADII.len() - 1);
    let radius = theme::SIGHT_RADII[step];
    if blocks.view_radius != radius {
        blocks.view_radius = radius;
        blocks.keep_radius = radius + 2;
    }
}
