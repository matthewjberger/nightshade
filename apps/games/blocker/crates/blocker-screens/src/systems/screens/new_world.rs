use crate::resources::{CatalogPanel, NewWorldPanel, TitlePanel};
use crate::theme;
use blocker_blocks::resources::VoxelWorld;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_save::resources::Save;
use blocker_screen::data::screen::Screen;
use blocker_world::commands::{set_all_categories, toggle_category};
use blocker_world::data::catalog::CATEGORIES;
use blocker_world::queries::{category_on, fresh_seed};
use blocker_world::resources::Draft;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_action_button, add_body_column, add_card, add_cell_text, add_footer_buttons,
    add_row_strip, add_scrolling_body, add_section_label, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_new_world(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut new_world_panel: ResMut<NewWorldPanel>,
    mut catalog_panel: ResMut<CatalogPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    new_world_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::NEW_WORLD_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = new_world_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "A new world");
        let body = add_body_column(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            theme::NEW_WORLD_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            let scroll = add_scrolling_body(
                tree,
                theme::NEW_WORLD_HEIGHT
                    - blocker_hud::theme::TITLE_HEIGHT
                    - blocker_hud::theme::FOOTER_HEIGHT
                    - blocker_hud::theme::BODY_PADDING * 2.0,
            );
            tree.in_parent(scroll, |tree| {
                add_section_label(
                    tree,
                    &style,
                    "The ground the world is made of",
                    blocker_hud::theme::ROW_WIDTH,
                );
                let strip = add_row_strip(tree, &style, 0, blocker_hud::theme::ROW_WIDTH);
                tree.in_parent(strip, |tree| {
                    add_cell_text(
                        tree,
                        &style,
                        "The number the world grows from",
                        theme::SETTING_NAME_WIDTH - theme::SEED_WIDTH,
                        blocker_hud::theme::BODY_SIZE,
                        blocker_hud::theme::TEXT_COLOR,
                    );
                    new_world_panel.seed_label = add_cell_text(
                        tree,
                        &style,
                        "",
                        theme::SEED_WIDTH,
                        blocker_hud::theme::BODY_SIZE,
                        blocker_hud::theme::ACCENT_COLOR,
                    );
                    new_world_panel.seed_button = add_action_button(tree, &style, "Roll");
                });
                add_section_label(
                    tree,
                    &style,
                    "What the world is filled with. It starts empty.",
                    blocker_hud::theme::ROW_WIDTH,
                );
                let all_strip = add_row_strip(tree, &style, 0, blocker_hud::theme::ROW_WIDTH);
                catalog_panel.all = tree.in_parent(all_strip, |tree| {
                    add_cell_text(
                        tree,
                        &style,
                        "Everything",
                        theme::SETTING_NAME_WIDTH,
                        blocker_hud::theme::BODY_SIZE,
                        blocker_hud::theme::TEXT_COLOR,
                    );
                    add_action_button(tree, &style, "Toggle all")
                });
                let mut labels = Vec::new();
                let mut toggles = Vec::new();
                for (index, category) in CATEGORIES.iter().enumerate() {
                    let strip = add_row_strip(tree, &style, index, blocker_hud::theme::ROW_WIDTH);
                    let (state, button) = tree.in_parent(strip, |tree| {
                        add_cell_text(
                            tree,
                            &style,
                            category.name,
                            theme::SETTING_NAME_WIDTH,
                            blocker_hud::theme::BODY_SIZE,
                            blocker_hud::theme::TEXT_COLOR,
                        );
                        let state = add_cell_text(
                            tree,
                            &style,
                            "off",
                            theme::SEED_WIDTH,
                            blocker_hud::theme::BODY_SIZE,
                            blocker_hud::theme::DIM_TEXT_COLOR,
                        );
                        let button = add_action_button(tree, &style, "Toggle");
                        (state, button)
                    });
                    labels.push(state);
                    toggles.push(button);
                    add_section_label(tree, &style, category.blurb, blocker_hud::theme::ROW_WIDTH);
                }
                catalog_panel.labels = labels;
                catalog_panel.buttons = toggles;
            });
        });
        let buttons = add_footer_buttons(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            &["Make the world", "Back"],
        );
        new_world_panel.make = buttons[0];
        new_world_panel.back = buttons[1];
    });
    menu_show_on(&mut menus, Screen::NewWorld, new_world_panel.panel);
    built_tree.finish();
}

pub fn show_the_catalog(
    draft: Res<Draft>,
    mut catalog_panel: ResMut<CatalogPanel>,
    mut commands: Commands,
) {
    if catalog_panel.labels.len() < CATEGORIES.len() {
        return;
    }
    if catalog_panel.shown == Some(draft.enabled) {
        return;
    }
    catalog_panel.shown = Some(draft.enabled);
    for index in 0..CATEGORIES.len() {
        let on = category_on(draft.enabled, index);
        ui_set_text(
            &mut commands,
            catalog_panel.labels[index],
            if on { "on" } else { "off" },
        );
        ui_set_color::<UiBase>(
            &mut commands,
            catalog_panel.labels[index],
            if on {
                blocker_hud::theme::ACCENT_COLOR
            } else {
                blocker_hud::theme::DIM_TEXT_COLOR
            },
        );
    }
}

pub fn show_the_draft(
    draft: Res<Draft>,
    save: Res<Save>,
    world: Res<VoxelWorld>,
    mut new_world_panel: ResMut<NewWorldPanel>,
    shell: Res<ShellHandles>,
    title_panel: Res<TitlePanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if shell.root == Entity::default() {
        return;
    }
    let found = save.found.is_some() || !world.chunks.is_empty();
    if new_world_panel.shown_found != Some(found) {
        new_world_panel.shown_found = Some(found);
        let button = title_panel.resume;
        ui_set_visible(&mut dirty, &mut commands, button, found);
    }
    if new_world_panel.shown_seed == Some(draft.seed) {
        return;
    }
    new_world_panel.shown_seed = Some(draft.seed);
    ui_set_text(
        &mut commands,
        new_world_panel.seed_label,
        &draft.seed.to_string(),
    );
}

pub fn new_world_buttons(
    catalog_panel: Res<CatalogPanel>,
    new_world_panel: Res<NewWorldPanel>,
    mut draft: ResMut<Draft>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, new_world_panel.back) {
        next.pending = Some(Screen::Title);
        return;
    }
    if ui_clicked(&mut buttons, new_world_panel.make) {
        draft.asked = true;
        return;
    }
    if ui_clicked(&mut buttons, new_world_panel.seed_button) {
        draft.seed = fresh_seed();
    }
    if ui_clicked(&mut buttons, catalog_panel.all) {
        let all_on = (0..CATEGORIES.len()).all(|index| category_on(draft.enabled, index));
        set_all_categories(&mut draft, !all_on);
    }
    for index in 0..catalog_panel.buttons.len().min(CATEGORIES.len()) {
        if ui_clicked(&mut buttons, catalog_panel.buttons[index]) {
            toggle_category(&mut draft, index);
        }
    }
}
