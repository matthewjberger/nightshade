use crate::resources::SetupPanel;
use crate::theme;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_player::resources::Profile;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::{
    CardStyle, add_body_column, add_card, add_footer_buttons, add_section_label, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_setup(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut setup_panel: ResMut<SetupPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    setup_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::SETUP_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    tree.in_parent(setup_panel.panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "Who are you?");
        let body = add_body_column(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            theme::SETUP_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "The name other travelers will know you by.",
                blocker_hud::theme::ROW_WIDTH,
            );
            setup_panel.name_input = tree.add_text_input(blocker_player::data::body::PLAYER_NAME);
        });
        let buttons = add_footer_buttons(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            &["Enter the world"],
        );
        setup_panel.play = buttons[0];
    });
    menu_show_on(&mut menus, Screen::Setup, setup_panel.panel);
    built_tree.finish();
}

pub fn setup_buttons(
    setup_panel: Res<SetupPanel>,
    mut profile: ResMut<Profile>,
    mut next: ResMut<NextState<Screen>>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut buttons: Query<(&UiButtonData,)>,
    mut inputs: Query<(&UiTextInputData,)>,
) {
    if !ui_clicked(&mut buttons, setup_panel.play) {
        return;
    }
    let mut name = String::new();
    inputs.for_entity(setup_panel.name_input, |(data,)| {
        name = data.text.trim().to_string();
    });
    if name.is_empty() {
        name = blocker_player::data::body::PLAYER_NAME.to_string();
    }
    profile.name = name;
    interaction.focused_entity = None;
    next.pending = Some(Screen::Title);
}
