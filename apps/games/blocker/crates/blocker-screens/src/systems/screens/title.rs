use crate::resources::Screens;
use crate::resources::TitlePanel;
use crate::theme;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_screen::data::screen::Screen;
use blocker_world::resources::{Draft, Generation};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::{
    CardStyle, add_body_column, add_card, add_footer_buttons, add_section_label, add_title_bar,
    add_wide_button,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_title(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut title_panel: ResMut<TitlePanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    title_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::TITLE_MENU_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = title_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            blocker_hud::theme::TITLE,
        );
        let body = add_body_column(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            theme::TITLE_MENU_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "Dig it, build on it, and keep it on the disk",
                blocker_hud::theme::ROW_WIDTH,
            );
            title_panel.resume = add_wide_button(
                tree,
                &style,
                blocker_hud::theme::ROW_WIDTH,
                "Go on with your world",
            );
            title_panel.new = add_wide_button(
                tree,
                &style,
                blocker_hud::theme::ROW_WIDTH,
                "Make a new world",
            );
            title_panel.settings =
                add_wide_button(tree, &style, blocker_hud::theme::ROW_WIDTH, "Settings");
        });
        title_panel.leave =
            add_footer_buttons(tree, &style, blocker_hud::theme::MENU_WIDTH, &["Leave"])[0];
    });
    menu_show_on(&mut menus, Screen::Title, title_panel.panel);
    built_tree.finish();
}

pub fn title_buttons(
    title_panel: Res<TitlePanel>,
    generation: Res<Generation>,
    mut draft: ResMut<Draft>,
    mut window: ResMut<Window>,
    mut screens: ResMut<Screens>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, title_panel.resume) {
        next.pending = Some(Screen::Loading);
        return;
    }
    if ui_clicked(&mut buttons, title_panel.new) {
        draft.seed = generation.seed;
        next.pending = Some(Screen::NewWorld);
        return;
    }
    if ui_clicked(&mut buttons, title_panel.settings) {
        screens.return_to = Screen::Title;
        next.pending = Some(Screen::Settings);
        return;
    }
    if ui_clicked(&mut buttons, title_panel.leave) {
        window.should_exit = true;
    }
}
