use crate::resources::CharacterPanel;
use crate::theme;
use blocker_beasts::data::beasts::BEASTS;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_player::components::Food;
use blocker_player::resources::Profile;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use blocker_quests::queries::{level_from_xp, xp_into_level};
use blocker_quests::resources::Journal;
use blocker_screen::data::screen::Screen;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_health::Health;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_bar, add_card, add_footer_buttons, add_section_label, add_stat_row,
    add_title_bar, set_bar_fill,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

const STATS: [&str; 7] = [
    "Name",
    "Level",
    "Coin",
    "Renown",
    "Quests done",
    "Items carried",
    "In hand",
];

pub fn build_character(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut character_panel: ResMut<CharacterPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    character_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        theme::CHARACTER_WIDTH,
        theme::CHARACTER_HEIGHT,
        theme::CLEAR_COLOR,
    );
    tree.in_parent(character_panel.panel, |tree| {
        let column = theme::CHARACTER_HEIGHT
            - blocker_hud::theme::TITLE_HEIGHT
            - blocker_hud::theme::FOOTER_HEIGHT;
        let stand_top = blocker_hud::theme::TITLE_HEIGHT + blocker_hud::theme::BODY_PADDING;
        let stand_bottom = stand_top + column - 20.0;
        let stand_right = blocker_hud::theme::BODY_PADDING + theme::CHARACTER_STAND_WIDTH;
        for (left, top, wide, tall) in [
            (0.0, 0.0, theme::CHARACTER_WIDTH, stand_top),
            (
                0.0,
                stand_top,
                blocker_hud::theme::BODY_PADDING,
                column - 20.0,
            ),
            (
                stand_right,
                stand_top,
                theme::CHARACTER_WIDTH - stand_right,
                column - 20.0,
            ),
            (
                0.0,
                stand_bottom,
                theme::CHARACTER_WIDTH,
                theme::CHARACTER_HEIGHT - stand_bottom,
            ),
        ] {
            tree.add_node()
                .window(Ab(vec2(left, top)), Ab(vec2(wide, tall)), Anchor::TopLeft)
                .with_rect(0.0, 0.0, theme::CLEAR_COLOR)
                .color_raw::<UiBase>(blocker_hud::theme::PANEL_COLOR);
        }
        add_title_bar(tree, &style, theme::CHARACTER_WIDTH, "Your character");
        let split = tree
            .add_node()
            .size_px(theme::CHARACTER_WIDTH, column)
            .flow(
                FlowDirection::Horizontal,
                blocker_hud::theme::BODY_PADDING,
                theme::CHARACTER_GAP,
            )
            .entity();
        tree.in_parent(split, |tree| {
            build_stand(tree, &mut character_panel, column - 20.0);
            build_stats(&style, tree, &mut character_panel, column - 20.0);
            build_slain(&style, tree, &mut character_panel, column - 20.0);
        });
        let buttons = add_footer_buttons(tree, &style, theme::CHARACTER_WIDTH, &["Close"]);
        character_panel.close = buttons[0];
    });
    menu_show_on(&mut menus, Screen::Character, character_panel.panel);
    built_tree.finish();
}

fn build_stand(
    tree: &mut UiTreeBuilder<'_, '_>,
    character_panel: &mut CharacterPanel,
    height: f32,
) {
    let stand = tree
        .add_node()
        .size_px(theme::CHARACTER_STAND_WIDTH, height)
        .with_rect(8.0, 1.0, blocker_hud::theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::CLEAR_COLOR)
        .entity();
    tree.in_parent(stand, |tree| {
        character_panel.caption = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::CHARACTER_CAPTION + 4.0)),
                Ab(vec2(theme::CHARACTER_STAND_WIDTH - 12.0, 20.0)),
                Anchor::TopCenter,
            )
            .with_text("", blocker_hud::theme::BODY_SIZE)
            .with_text_overflow(TextOverflow::Ellipsis)
            .text_center()
            .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
            .entity();
        character_panel.rank = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::CHARACTER_CAPTION + 24.0)),
                Ab(vec2(theme::CHARACTER_STAND_WIDTH - 12.0, 18.0)),
                Anchor::TopCenter,
            )
            .with_text("", blocker_hud::theme::SMALL_SIZE)
            .text_center()
            .color_raw::<UiBase>(blocker_hud::theme::DIM_TEXT_COLOR)
            .entity();
    });
}

fn build_stats(
    style: &CardStyle,
    tree: &mut UiTreeBuilder<'_, '_>,
    character_panel: &mut CharacterPanel,
    height: f32,
) {
    let body = tree
        .add_node()
        .size_px(theme::CHARACTER_STAT_WIDTH, height)
        .with_rect(8.0, 0.0, theme::CLEAR_COLOR)
        .color_raw::<UiBase>(theme::COLUMN_COLOR)
        .flow(FlowDirection::Vertical, 6.0, blocker_hud::theme::ROW_GAP)
        .entity();
    tree.in_parent(body, |tree| {
        let width = theme::CHARACTER_STAT_WIDTH - 12.0;
        let bars = tree
            .add_node()
            .size_px(width, theme::CHARACTER_BARS_HEIGHT)
            .entity();
        tree.in_parent(bars, |tree| {
            let bar_width = width - 12.0;
            let step = blocker_hud::theme::BAR_HEIGHT + 6.0;
            let (fill, text) = add_bar(
                tree,
                style,
                6.0,
                4.0,
                bar_width,
                blocker_hud::theme::HEALTH_COLOR,
            );
            character_panel.health_fill = fill;
            character_panel.health_text = text;
            let (fill, text) = add_bar(
                tree,
                style,
                6.0,
                4.0 + step,
                bar_width,
                blocker_hud::theme::FOOD_COLOR,
            );
            character_panel.food_fill = fill;
            character_panel.food_text = text;
            let (fill, text) = add_bar(
                tree,
                style,
                6.0,
                4.0 + step * 2.0,
                bar_width,
                blocker_hud::theme::XP_COLOR,
            );
            character_panel.xp_fill = fill;
            character_panel.xp_text = text;
        });
        character_panel.rows.clear();
        for (index, label) in STATS.iter().enumerate() {
            character_panel.rows.push(add_stat_row(
                tree,
                style,
                index,
                label,
                width,
                theme::CHARACTER_NAME_WIDTH,
            ));
        }
    });
}

fn build_slain(
    style: &CardStyle,
    tree: &mut UiTreeBuilder<'_, '_>,
    character_panel: &mut CharacterPanel,
    height: f32,
) {
    let body = tree
        .add_node()
        .size_px(theme::CHARACTER_SLAIN_WIDTH, height)
        .with_rect(8.0, 0.0, theme::CLEAR_COLOR)
        .color_raw::<UiBase>(theme::COLUMN_COLOR)
        .flow(FlowDirection::Vertical, 6.0, blocker_hud::theme::ROW_GAP)
        .entity();
    tree.in_parent(body, |tree| {
        add_section_label(
            tree,
            style,
            "Beasts slain",
            theme::CHARACTER_SLAIN_WIDTH - 12.0,
        );
        for (offset, beast) in BEASTS.iter().enumerate() {
            character_panel.rows.push(add_stat_row(
                tree,
                style,
                STATS.len() + offset,
                beast.name,
                theme::CHARACTER_SLAIN_WIDTH - 12.0,
                theme::CHARACTER_SLAIN_NAME_WIDTH,
            ));
        }
    });
}

pub fn show_the_character(
    style: Res<CardStyle>,
    profile: Res<Profile>,
    journal: Res<Journal>,
    inventory: Res<Inventory>,
    hero: Res<Hero>,
    mut character_panel: ResMut<CharacterPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut bodies: Query<(&Health, &Food)>,
) {
    if character_panel.rows.is_empty() {
        return;
    }
    let Some((body_health, body_food)) =
        read_the_hero(&hero, &mut bodies, |(health, food)| (health.0, food.0))
    else {
        return;
    };
    let carried: usize = inventory
        .slots
        .0
        .iter()
        .map(|slot| slot.count as usize)
        .sum();
    let held = selected_stack(&inventory)
        .map(|stack| stack.item.name())
        .unwrap_or("Nothing");
    let level = level_from_xp(journal.xp);
    let (into, span) = xp_into_level(journal.xp);
    let mut values = vec![
        profile.name.to_string(),
        format!("{level}"),
        format!("{}", journal.coin),
        format!("{}", journal.renown),
        format!("{}", journal.finished.len()),
        format!("{carried}"),
        held.to_string(),
    ];
    for count in &journal.slain {
        values.push(format!("{count}"));
    }
    let health = body_health.round() as i32;
    let food = body_food.round() as i32;
    let key = (values, health, food);
    if character_panel.shown.as_ref() == Some(&key) {
        return;
    }
    let values = &key.0;
    character_panel.shown = Some(key.clone());
    for (row, value) in character_panel.rows.iter().zip(values.iter()) {
        ui_set_text(&mut commands, *row, value);
    }
    ui_set_text(&mut commands, character_panel.caption, &profile.name);
    ui_set_text(
        &mut commands,
        character_panel.rank,
        &format!(
            "Level {level}, {} of the watch",
            rank_word(journal.finished.len())
        ),
    );
    let bar_width = theme::CHARACTER_STAT_WIDTH - 24.0;
    set_bar_fill(
        &mut dirty,
        &mut commands,
        &style,
        character_panel.health_fill,
        bar_width,
        body_health / blocker_player::data::body::FULL_HEALTH,
    );
    ui_set_text(
        &mut commands,
        character_panel.health_text,
        &format!(
            "Health {health} / {}",
            blocker_player::data::body::FULL_HEALTH.round() as i32
        ),
    );
    set_bar_fill(
        &mut dirty,
        &mut commands,
        &style,
        character_panel.food_fill,
        bar_width,
        body_food / blocker_player::data::body::FULL_FOOD,
    );
    ui_set_text(
        &mut commands,
        character_panel.food_text,
        &format!(
            "Food {food} / {}",
            blocker_player::data::body::FULL_FOOD.round() as i32
        ),
    );
    let fraction = if span == 0 {
        1.0
    } else {
        into as f32 / span as f32
    };
    set_bar_fill(
        &mut dirty,
        &mut commands,
        &style,
        character_panel.xp_fill,
        bar_width,
        fraction,
    );
    ui_set_text(
        &mut commands,
        character_panel.xp_text,
        &format!("Experience {into} / {span}"),
    );
}

fn rank_word(finished: usize) -> &'static str {
    match finished {
        0 => "recruit",
        1..=2 => "hand",
        3..=5 => "blade",
        6..=9 => "sergeant's pick",
        _ => "named",
    }
}

pub fn character_buttons(
    character_panel: Res<CharacterPanel>,
    screens: Res<crate::resources::Screens>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, character_panel.close) {
        next.pending = Some(screens.return_to);
    }
}
