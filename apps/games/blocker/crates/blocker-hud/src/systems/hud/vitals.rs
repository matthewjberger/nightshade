use crate::resources::HudBuild;
use crate::resources::{ShellHandles, VitalsPanel};
use crate::theme;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_player::components::Food;
use blocker_player::resources::Profile;
use blocker_quests::queries::{level_from_xp, xp_into_level};
use blocker_quests::resources::Journal;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_health::Health;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{CardStyle, add_bar, add_frame, set_bar_fill};

pub fn build_vitals(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    let panel = tree
        .card_node(8.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
        .window(
            Ab(vec2(theme::FRAME_MARGIN, theme::FRAME_MARGIN)),
            Ab(vec2(theme::FRAME_WIDTH, theme::FRAME_HEIGHT)),
            Anchor::TopLeft,
        )
        .entity();
    let bar_width = theme::FRAME_WIDTH - 24.0;
    let frame_layer = hud.art.frame;
    tree.in_parent(panel, |tree| {
        add_frame(tree, hud.style, frame_layer);
        hud.vitals.name_label = tree
            .add_node()
            .window(
                Ab(vec2(12.0, 8.0)),
                Ab(vec2(bar_width, 20.0)),
                Anchor::TopLeft,
            )
            .with_text(blocker_player::data::body::PLAYER_NAME, theme::BODY_SIZE)
            .text_left()
            .color_raw::<UiBase>(theme::UNIT_NAME_COLOR)
            .entity();
        let (fill, text) = add_bar(tree, hud.style, 12.0, 34.0, bar_width, theme::HEALTH_COLOR);
        hud.vitals.health_fill = fill;
        hud.vitals.health_text = text;
        let (fill, text) = add_bar(tree, hud.style, 12.0, 64.0, bar_width, theme::FOOD_COLOR);
        hud.vitals.food_fill = fill;
        hud.vitals.food_text = text;
        let (fill, text) = add_bar(tree, hud.style, 12.0, 94.0, bar_width, theme::XP_COLOR);
        hud.vitals.xp_fill = fill;
        hud.vitals.xp_text = text;
    });
}

pub fn show_the_vitals(
    style: Res<CardStyle>,
    journal: Res<Journal>,
    profile: Res<Profile>,
    shell: Res<ShellHandles>,
    hero: Res<Hero>,
    mut vitals_panel: ResMut<VitalsPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut bodies: Query<(&Health, &Food)>,
) {
    if shell.root == Entity::default() {
        return;
    }
    if vitals_panel.shown_name != profile.name {
        vitals_panel.shown_name = profile.name.clone();
        ui_set_text(&mut commands, vitals_panel.name_label, &profile.name);
    }
    let bar_width = theme::FRAME_WIDTH - 24.0;
    if vitals_panel.shown_xp != Some(journal.xp) {
        vitals_panel.shown_xp = Some(journal.xp);
        let level = level_from_xp(journal.xp);
        let (into, span) = xp_into_level(journal.xp);
        let fraction = if span == 0 {
            1.0
        } else {
            into as f32 / span as f32
        };
        set_bar_fill(
            &mut dirty,
            &mut commands,
            &style,
            vitals_panel.xp_fill,
            bar_width,
            fraction,
        );
        ui_set_text(
            &mut commands,
            vitals_panel.xp_text,
            &format!("Lv {level}   {into} / {span}"),
        );
    }
    let Some((body_health, body_food)) =
        read_the_hero(&hero, &mut bodies, |(health, food)| (health.0, food.0))
    else {
        return;
    };
    let health = body_health.max(0.0).ceil() as u32;
    if vitals_panel.shown_health != Some(health) {
        vitals_panel.shown_health = Some(health);
        set_bar_fill(
            &mut dirty,
            &mut commands,
            &style,
            vitals_panel.health_fill,
            bar_width,
            body_health / blocker_player::data::body::FULL_HEALTH,
        );
        ui_set_text(
            &mut commands,
            vitals_panel.health_text,
            &format!(
                "{health} / {}",
                blocker_player::data::body::FULL_HEALTH as u32
            ),
        );
    }
    let food = body_food.max(0.0).ceil() as u32;
    if vitals_panel.shown_food != Some(food) {
        vitals_panel.shown_food = Some(food);
        set_bar_fill(
            &mut dirty,
            &mut commands,
            &style,
            vitals_panel.food_fill,
            bar_width,
            body_food / blocker_player::data::body::FULL_FOOD,
        );
        ui_set_text(
            &mut commands,
            vitals_panel.food_text,
            &format!("{food} / {}", blocker_player::data::body::FULL_FOOD as u32),
        );
    }
}
