use crate::resources::HudBuild;
use crate::resources::{ShellHandles, TargetPanel};
use crate::theme;
use blocker_beasts::components::{Angry, Beast};
use blocker_beasts::data::beasts::BEASTS;
use blocker_beasts::resources::Quarry;
use nalgebra_glm::{vec2, vec4};
use nightshade::prelude::*;
use nightshade_health::Health;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{CardStyle, add_bar, add_frame, set_bar_fill};

pub fn build_target(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    let panel = tree
        .card_node(8.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
        .window(
            Ab(vec2(
                theme::FRAME_MARGIN,
                theme::FRAME_MARGIN + theme::FRAME_HEIGHT + theme::FRAME_GAP,
            )),
            Ab(vec2(theme::FRAME_WIDTH, theme::TARGET_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_visible(false)
        .entity();
    let frame_layer = hud.art.frame;
    let bar_width = theme::FRAME_WIDTH - 24.0;
    tree.in_parent(panel, |tree| {
        add_frame(tree, hud.style, frame_layer);
        hud.target.name = tree
            .add_node()
            .window(
                Ab(vec2(12.0, 8.0)),
                Ab(vec2(bar_width, 20.0)),
                Anchor::TopLeft,
            )
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::UNIT_NAME_COLOR)
            .entity();
        let (fill, text) = add_bar(
            tree,
            hud.style,
            12.0,
            34.0,
            bar_width,
            blocker_beasts::data::beasts::THREAT_OUTLINE_COLOR.into(),
        );
        hud.target.health_fill = fill;
        hud.target.health_text = text;
    });
    hud.target.panel = panel;
}

pub fn show_the_target(
    style: Res<CardStyle>,
    quarry: Res<Quarry>,
    shell: Res<ShellHandles>,
    mut target_panel: ResMut<TargetPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut beasts: Query<(&Beast, &Health, &Angry)>,
) {
    if shell.root == Entity::default() {
        return;
    }
    let bar_width = theme::FRAME_WIDTH - 24.0;
    let mut found: Option<(Entity, usize, f32, bool)> = None;
    if let Some(target) = quarry.0 {
        beasts.for_entity(target, |(beast, health, angry)| {
            found = Some((target, beast.0, health.0, angry.0 > 0.0));
        });
    }
    match found {
        Some((target, kind, health, hostile)) => {
            let current = health.max(0.0).ceil() as u32;
            let key = (target, current, hostile);
            if target_panel.shown != Some(key) {
                target_panel.shown = Some(key);
                let beast = &BEASTS[kind];
                let full = beast.health.ceil() as u32;
                set_bar_fill(
                    &mut dirty,
                    &mut commands,
                    &style,
                    target_panel.health_fill,
                    bar_width,
                    health / beast.health,
                );
                let tint = if hostile {
                    blocker_beasts::data::beasts::THREAT_OUTLINE_COLOR
                } else {
                    blocker_beasts::data::beasts::NEUTRAL_OUTLINE_COLOR
                };
                ui_set_color::<UiBase>(
                    &mut commands,
                    target_panel.health_fill,
                    vec4(tint[0], tint[1], tint[2], tint[3]),
                );
                ui_set_text(
                    &mut commands,
                    target_panel.health_text,
                    &format!("{current} / {full}"),
                );
                ui_set_text(&mut commands, target_panel.name, beast.name);
                ui_set_visible(&mut dirty, &mut commands, target_panel.panel, true);
            }
        }
        None => {
            if target_panel.shown.is_some() {
                target_panel.shown = None;
                ui_set_visible(&mut dirty, &mut commands, target_panel.panel, false);
            }
        }
    }
}
