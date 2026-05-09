use nightshade::ecs::ui::components::{ThemeEffect, ThemeShadow};
use nightshade::prelude::*;

const BLOCK_PADDING: f32 = 20.0;

pub fn build_showcase_screen(tree: &mut UiTreeBuilder, content: Entity) {
    tree.in_parent(content, |tree| {
        showcase_hero(tree);
        showcase_spacer(tree, 24.0);
        showcase_metric_row(tree);
        showcase_spacer(tree, 24.0);
        showcase_quick_actions(tree);
        showcase_spacer(tree, 24.0);
        showcase_split_section(tree);
    });
}

fn showcase_spacer(tree: &mut UiTreeBuilder, height: f32) {
    tree.add_node().size(100.pct(), (height).px()).entity();
}

fn showcase_card_inner(tree: &mut UiTreeBuilder, height: f32) -> Entity {
    let card = tree
        .add_node()
        .size(100.pct(), (height).px())
        .card()
        .entity();
    tree.in_parent(card, |tree| {
        tree.add_node()
            .boundary(
                Ab(vec2(BLOCK_PADDING, BLOCK_PADDING)),
                Ab(vec2(-BLOCK_PADDING, -BLOCK_PADDING)) + Rl(vec2(100.0, 100.0)),
            )
            .entity()
    })
}

fn showcase_hero(tree: &mut UiTreeBuilder) {
    let avatar_size = 64.0_f32;
    let middle_height = 76.0_f32;
    let action_height = 36.0_f32;
    let row_gap = 16.0_f32;
    let hero_height = BLOCK_PADDING * 2.0 + middle_height + row_gap + action_height;
    let inner = showcase_card_inner(tree, hero_height);
    tree.in_parent(inner, |tree| {
        let column = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .flow_vertical()
            .padding(0.0)
            .gap(row_gap)
            .entity();
        tree.in_parent(column, |tree| {
            let top_row = tree
                .add_node()
                .size(100.pct(), (avatar_size).px())
                .flow_horizontal()
                .padding(0.0)
                .gap(20.0)
                .align_cross(FlowAlignment::Center)
                .entity();
            tree.in_parent(top_row, |tree| {
                tree.add_node()
                    .size((avatar_size).px(), (avatar_size).px())
                    .rect(12.0)
                    .fg(ThemeColor::Accent)
                    .with_children(|child| {
                        child
                            .add_node()
                            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(28.0, 28.0)), Anchor::Center)
                            .with_icon(icons::material::AUTO_AWESOME, 30.0)
                            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0));
                    })
                    .entity();

                let middle = tree
                    .add_node()
                    .size((0.0).px(), (avatar_size).px())
                    .flex_grow(1.0)
                    .with_clip()
                    .entity();
                tree.in_parent(middle, |tree| {
                    tree.add_node()
                        .window(
                            Ab(vec2(0.0, 4.0)),
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("Aurora Atlas", 24.0)
                        .text_left()
                        .fg(ThemeColor::Text);

                    tree.add_node()
                        .window(
                            Ab(vec2(0.0, 34.0)),
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("Updated 2 hours ago by Mateo", 13.0)
                        .text_left()
                        .fg(ThemeColor::TextDisabled);

                    tree.add_node()
                        .window(Ab(vec2(0.0, 60.0)), Ab(vec2(8.0, 8.0)), Anchor::TopLeft)
                        .rect(4.0)
                        .fg(ThemeColor::Success);

                    tree.add_node()
                        .window(
                            Ab(vec2(16.0, 56.0)),
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 16.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("Live  -  3 collaborators online", 13.0)
                        .text_left()
                        .fg(ThemeColor::TextDisabled);
                });
            });

            let action_row = tree
                .add_node()
                .size(100.pct(), (action_height).px())
                .flow_horizontal()
                .padding(0.0)
                .gap(12.0)
                .align_main(FlowAlignment::End)
                .align_cross(FlowAlignment::Center)
                .flow_wrap()
                .entity();
            tree.in_parent(action_row, |tree| {
                showcase_secondary_button(tree, "Share project", 120.0);
                showcase_primary_button(tree, "Open editor", 220.0);
            });
        });
    });
}

fn showcase_metric_row(tree: &mut UiTreeBuilder) {
    let row = tree
        .add_node()
        .fill_width()
        .auto_size(AutoSizeMode::Height)
        .flow_horizontal()
        .padding(0.0)
        .gap(16.0)
        .flow_wrap()
        .entity();
    tree.in_parent(row, |tree| {
        showcase_metric_card(
            tree,
            icons::material::TABLE_CHART,
            "Active sessions",
            "1,284",
            "+12.4% vs last week",
            true,
        );
        showcase_metric_card(
            tree,
            icons::material::AUTO_AWESOME,
            "Conversion rate",
            "8.2%",
            "+0.6 pts vs last week",
            true,
        );
        showcase_metric_card(
            tree,
            icons::material::PALETTE,
            "Avg. response",
            "184ms",
            "-22ms vs last week",
            true,
        );
    });
}

fn showcase_metric_card(
    tree: &mut UiTreeBuilder,
    icon: IconGlyph,
    label: &str,
    value: &str,
    delta: &str,
    positive: bool,
) {
    let card = tree
        .add_node()
        .size((0.0).px(), (110.0).px())
        .flex_grow(1.0)
        .with_min_size(vec2(220.0, 110.0))
        .card()
        .entity();
    tree.in_parent(card, |tree| {
        let icon_tile = tree
            .add_node()
            .window(
                Ab(vec2(BLOCK_PADDING, BLOCK_PADDING)),
                Ab(vec2(28.0, 28.0)),
                Anchor::TopLeft,
            )
            .rect(8.0)
            .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
            .entity();
        tree.in_parent(icon_tile, |tree| {
            tree.add_node()
                .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .rect(8.0)
                .fg(ThemeColor::Accent)
                .with_theme_effect_role(ThemeEffect::PanelEffect)
                .entity();
            tree.add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(16.0, 16.0)), Anchor::Center)
                .with_icon(icon, 17.0)
                .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0))
                .entity();
        });

        tree.add_node()
            .window(
                Ab(vec2(BLOCK_PADDING + 40.0, BLOCK_PADDING)),
                Ab(vec2(180.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text(label, 14.0)
            .text_left()
            .fg(ThemeColor::TextDisabled);

        tree.add_node()
            .window(
                Ab(vec2(BLOCK_PADDING, BLOCK_PADDING + 32.0)),
                Ab(vec2(220.0, 30.0)),
                Anchor::TopLeft,
            )
            .with_text(value, 28.0)
            .text_left()
            .fg(ThemeColor::Text);

        let delta_role = if positive {
            ThemeColor::Success
        } else {
            ThemeColor::Error
        };
        tree.add_node()
            .window(
                Ab(vec2(BLOCK_PADDING, BLOCK_PADDING + 64.0)),
                Ab(vec2(220.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text(delta, 12.0)
            .text_left()
            .fg(delta_role);
    });
}

fn showcase_quick_actions(tree: &mut UiTreeBuilder) {
    let theme = tree.active_theme().clone();
    let row_height = 88.0;
    let inner = showcase_card_inner(tree, row_height);
    tree.in_parent(inner, |tree| {
        tree.add_node()
            .window(Ab(vec2(0.0, 4.0)), Ab(vec2(180.0, 16.0)), Anchor::TopLeft)
            .with_text("Quick actions", 15.0)
            .text_left()
            .fg(ThemeColor::Text);

        let row = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 28.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 36.0)),
                Anchor::TopLeft,
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(12.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(row, |tree| {
            let actions = [
                (icons::material::KEYBOARD, "New document"),
                (icons::material::TOGGLE_ON, "Invite member"),
                (icons::material::TABLE_CHART, "Open report"),
                (icons::material::TUNE, "Settings"),
            ];
            for (icon, label) in actions {
                showcase_action_chip(
                    tree,
                    icon,
                    label,
                    theme.text_color,
                    theme.text_color_disabled,
                );
            }
        });
    });
}

fn showcase_action_chip(
    tree: &mut UiTreeBuilder,
    icon: IconGlyph,
    label: &str,
    text_color: nalgebra_glm::Vec4,
    icon_color: nalgebra_glm::Vec4,
) {
    let theme = tree.active_theme().clone();
    let chip = tree
        .add_node()
        .flow_child(Rl(vec2(0.0, 100.0)))
        .flex_grow(1.0)
        .with_rect(8.0, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .fg(ThemeColor::Background)
        .fg_hover(ThemeColor::BackgroundHover)
        .fg_pressed(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_state_offset::<UiHover>(vec2(0.0, -1.0))
        .with_state_offset::<UiPressed>(vec2(0.0, 1.0))
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.in_parent(chip, |tree| {
        tree.add_node()
            .window(
                Ab(vec2(12.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(16.0, 16.0)),
                Anchor::CenterLeft,
            )
            .with_icon(icon, 15.0)
            .color_raw::<UiBase>(icon_color)
            .entity();
        tree.add_node()
            .window(
                Ab(vec2(34.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(160.0, 16.0)),
                Anchor::CenterLeft,
            )
            .with_text(label, 14.0)
            .text_left()
            .color_raw::<UiBase>(text_color)
            .entity();
    });
}

fn showcase_split_section(tree: &mut UiTreeBuilder) {
    let row = tree
        .add_node()
        .fill_width()
        .auto_size(AutoSizeMode::Height)
        .flow_horizontal()
        .padding(0.0)
        .gap(16.0)
        .flow_wrap()
        .entity();
    tree.in_parent(row, |tree| {
        showcase_activity_card(tree);
        showcase_settings_card(tree);
    });
}

fn showcase_activity_card(tree: &mut UiTreeBuilder) {
    let theme = tree.active_theme().clone();
    let card = tree
        .add_node()
        .size((0.0).px(), (280.0).px())
        .flex_grow(1.6)
        .with_min_size(vec2(320.0, 280.0))
        .card()
        .entity();
    tree.in_parent(card, |tree| {
        let inner = tree
            .add_node()
            .boundary(
                Ab(vec2(BLOCK_PADDING, BLOCK_PADDING)),
                Ab(vec2(-BLOCK_PADDING, -BLOCK_PADDING)) + Rl(vec2(100.0, 100.0)),
            )
            .flow_vertical()
            .padding(0.0)
            .gap(10.0)
            .entity();
        tree.in_parent(inner, |tree| {
            tree.add_node()
                .size(100.pct(), (22.0).px())
                .with_text("Recent activity", 16.0)
                .text_left()
                .fg(ThemeColor::Text)
                .entity();

            let entries = [
                (
                    "Mateo",
                    "edited Aurora Atlas / scenes / main.scn",
                    "2m ago",
                    theme.accent_color,
                ),
                (
                    "Lin",
                    "merged a pull request from playtest/balance",
                    "12m ago",
                    theme.success_color,
                ),
                (
                    "Build",
                    "completed in 1m 42s on origin/main",
                    "28m ago",
                    theme.text_color_disabled,
                ),
                (
                    "Sora",
                    "added 18 frames to character/run cycle",
                    "1h ago",
                    theme.warning_color,
                ),
                (
                    "Wren",
                    "shared a clip with the playtest channel",
                    "3h ago",
                    theme.text_color_accent,
                ),
            ];
            for (who, message, when, accent) in entries {
                let row = tree
                    .add_node()
                    .size(100.pct(), (28.0).px())
                    .flow_horizontal()
                    .padding(0.0)
                    .gap(10.0)
                    .align_cross(FlowAlignment::Center)
                    .entity();
                tree.in_parent(row, |tree| {
                    tree.add_node()
                        .size((8.0).px(), (8.0).px())
                        .rect(4.0)
                        .color_raw::<UiBase>(accent)
                        .entity();

                    tree.add_node()
                        .size((60.0).px(), (18.0).px())
                        .with_text(who, 13.0)
                        .text_left()
                        .fg(ThemeColor::Text)
                        .entity();

                    tree.add_node()
                        .size((0.0).px(), (18.0).px())
                        .flex_grow(1.0)
                        .with_text(message, 13.0)
                        .text_left()
                        .fg(ThemeColor::TextDisabled)
                        .entity();

                    tree.add_node()
                        .size((60.0).px(), (18.0).px())
                        .with_text(when, 12.0)
                        .text_right()
                        .fg(ThemeColor::TextDisabled)
                        .entity();
                });
            }
        });
    });
}

fn showcase_settings_card(tree: &mut UiTreeBuilder) {
    let card = tree
        .add_node()
        .size((0.0).px(), (280.0).px())
        .flex_grow(1.0)
        .with_min_size(vec2(280.0, 280.0))
        .card()
        .entity();
    tree.in_parent(card, |tree| {
        let inner = tree
            .add_node()
            .boundary(
                Ab(vec2(BLOCK_PADDING, BLOCK_PADDING)),
                Ab(vec2(-BLOCK_PADDING, -BLOCK_PADDING)) + Rl(vec2(100.0, 100.0)),
            )
            .flow_vertical()
            .gap(14.0)
            .entity();
        tree.in_parent(inner, |tree| {
            tree.add_node()
                .size(100.pct(), (22.0).px())
                .with_text("Workspace", 16.0)
                .text_left()
                .fg(ThemeColor::Text)
                .entity();

            showcase_setting_row(tree, "Visibility", |t| {
                t.add_dropdown(&["Private", "Team", "Public"], 1);
            });
            showcase_setting_row(tree, "Auto-save", |t| {
                t.add_toggle(true);
            });
            showcase_setting_row(tree, "Quality", |t| {
                t.add_slider(0.0, 100.0, 80.0);
            });
            showcase_setting_row(tree, "Density", |t| {
                t.add_drag_value_configured(DragValueConfig::new(0.5, 2.0, 1.0).speed(0.01));
            });
        });
    });
}

fn showcase_setting_row(
    tree: &mut UiTreeBuilder,
    label: &str,
    body: impl FnOnce(&mut UiTreeBuilder),
) {
    let row = tree
        .add_node()
        .size(100.pct(), (32.0).px())
        .flow_horizontal()
        .padding(0.0)
        .gap(12.0)
        .align_cross(FlowAlignment::Center)
        .entity();
    tree.in_parent(row, |tree| {
        tree.add_node()
            .size((96.0).px(), (18.0).px())
            .with_text(label, 14.0)
            .text_left()
            .fg(ThemeColor::TextDisabled)
            .entity();
        let widget_holder = tree
            .add_node()
            .size((0.0).px(), (32.0).px())
            .flex_grow(1.0)
            .flow_horizontal()
            .padding(0.0)
            .gap(8.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(widget_holder, body);
    });
}

fn showcase_primary_button(tree: &mut UiTreeBuilder, label: &str, width: f32) -> Entity {
    let theme = tree.active_theme().clone();
    let entity = tree
        .add_node()
        .size((width).px(), (36.0).px())
        .rect(8.0)
        .color_raw::<UiBase>(theme.accent_color)
        .color_raw::<UiHover>(theme.accent_color_hovered)
        .color_raw::<UiPressed>(theme.accent_color_active)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_state_offset::<UiHover>(vec2(0.0, -1.0))
        .with_state_offset::<UiPressed>(vec2(0.0, 1.0))
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.in_parent(entity, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(width - 16.0, 22.0)),
                Anchor::Center,
            )
            .with_text(label, 14.0)
            .text_center()
            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0))
            .entity();
    });
    entity
}

fn showcase_secondary_button(tree: &mut UiTreeBuilder, label: &str, width: f32) -> Entity {
    let theme = tree.active_theme().clone();
    let entity = tree
        .add_node()
        .size((width).px(), (36.0).px())
        .with_rect(8.0, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .fg(ThemeColor::Background)
        .fg_hover(ThemeColor::BackgroundHover)
        .fg_pressed(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_state_offset::<UiHover>(vec2(0.0, -1.0))
        .with_state_offset::<UiPressed>(vec2(0.0, 1.0))
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.in_parent(entity, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(width - 16.0, 22.0)),
                Anchor::Center,
            )
            .with_text(label, 14.0)
            .text_center()
            .fg(ThemeColor::Text)
            .entity();
    });
    entity
}
