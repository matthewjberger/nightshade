use nightshade::ecs::ui::components::{ThemeEffect, ThemeShadow};
use nightshade::prelude::*;

const BLOCK_PADDING: f32 = 20.0;
const BLOCK_RADIUS: f32 = 8.0;

pub fn build_showcase_screen(tree: &mut UiTreeBuilder, screen: Entity, content: Entity) {
    tree.push_parent(content);
    showcase_hero(tree);
    showcase_spacer(tree, 24.0);
    showcase_metric_row(tree);
    showcase_spacer(tree, 24.0);
    showcase_quick_actions(tree);
    showcase_spacer(tree, 24.0);
    showcase_split_section(tree);
    tree.pop_parent();
    let _ = screen;
}

fn showcase_spacer(tree: &mut UiTreeBuilder, height: f32) {
    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, height)),
        )
        .without_pointer_events()
        .done();
}

fn showcase_card_inner(tree: &mut UiTreeBuilder, height: f32) -> Entity {
    let theme = tree.active_theme().clone();
    let card = tree
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, height)),
        )
        .with_rect(BLOCK_RADIUS, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_theme_effect_role(ThemeEffect::PanelEffect)
        .entity();
    tree.push_parent(card);
    let inner = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING)),
            Ab(nalgebra_glm::Vec2::new(-BLOCK_PADDING, -BLOCK_PADDING))
                + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .without_pointer_events()
        .entity();
    tree.pop_parent();
    inner
}

fn showcase_hero(tree: &mut UiTreeBuilder) {
    let theme = tree.active_theme().clone();
    let hero_height = 120.0;
    let inner = showcase_card_inner(tree, hero_height);
    tree.push_parent(inner);

    let avatar_size = 64.0;
    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(0.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(avatar_size, avatar_size)),
            Anchor::CenterLeft,
        )
        .with_rect(12.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(theme.accent_color)
        .with_children(|child| {
            child
                .add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(28.0, 28.0)),
                    Anchor::Center,
                )
                .with_icon(icons::material::AUTO_AWESOME, 28.0)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0))
                .without_pointer_events();
        })
        .without_pointer_events()
        .done();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(avatar_size + 20.0, 18.0)),
            Ab(nalgebra_glm::Vec2::new(360.0, 26.0)),
            Anchor::TopLeft,
        )
        .with_text("Aurora Atlas", 22.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(avatar_size + 20.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(420.0, 18.0)),
            Anchor::TopLeft,
        )
        .with_text("Updated 2 hours ago by Mateo", 12.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(avatar_size + 20.0, 76.0)),
            Ab(nalgebra_glm::Vec2::new(8.0, 8.0)),
            Anchor::TopLeft,
        )
        .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(theme.success_color)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(avatar_size + 36.0, 72.0)),
            Ab(nalgebra_glm::Vec2::new(220.0, 16.0)),
            Anchor::TopLeft,
        )
        .with_text("Live  -  3 collaborators online", 12.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events();

    let action_y = (hero_height - 36.0 - BLOCK_PADDING * 2.0) * 0.5;
    let action_holder = tree
        .add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                + Ab(nalgebra_glm::Vec2::new(-(220.0 + 12.0 + 120.0), action_y)),
            Ab(nalgebra_glm::Vec2::new(220.0 + 12.0 + 120.0, 36.0)),
            Anchor::TopLeft,
        )
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            12.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(action_holder);
    showcase_secondary_button(tree, "Share project", 120.0);
    showcase_primary_button(tree, "Open editor", 220.0);
    tree.pop_parent();

    tree.pop_parent();
}

fn showcase_metric_row(tree: &mut UiTreeBuilder) {
    let row_height = 110.0;
    let row = tree
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, row_height)),
        )
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            16.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(row);
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
    tree.pop_parent();
}

fn showcase_metric_card(
    tree: &mut UiTreeBuilder,
    icon: IconGlyph,
    label: &str,
    value: &str,
    delta: &str,
    positive: bool,
) {
    let theme = tree.active_theme().clone();
    let card = tree
        .add_node()
        .flow_child(Rl(nalgebra_glm::Vec2::new(0.0, 100.0)))
        .flex_grow(1.0)
        .with_rect(BLOCK_RADIUS, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_theme_effect_role(ThemeEffect::PanelEffect)
        .entity();
    tree.push_parent(card);

    let icon_bg = nalgebra_glm::Vec4::new(
        theme.accent_color.x,
        theme.accent_color.y,
        theme.accent_color.z,
        0.18,
    );
    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING)),
            Ab(nalgebra_glm::Vec2::new(28.0, 28.0)),
            Anchor::TopLeft,
        )
        .with_rect(8.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(icon_bg)
        .with_children(|child| {
            child
                .add_node()
                .window(
                    Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                    Ab(nalgebra_glm::Vec2::new(16.0, 16.0)),
                    Anchor::Center,
                )
                .with_icon(icon, 16.0)
                .with_color::<UiBase>(theme.accent_color)
                .without_pointer_events();
        })
        .without_pointer_events()
        .done();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING + 40.0, BLOCK_PADDING)),
            Ab(nalgebra_glm::Vec2::new(180.0, 16.0)),
            Anchor::TopLeft,
        )
        .with_text(label, 12.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING + 32.0)),
            Ab(nalgebra_glm::Vec2::new(220.0, 30.0)),
            Anchor::TopLeft,
        )
        .with_text(value, 26.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events();

    let delta_color = if positive {
        theme.success_color
    } else {
        theme.error_color
    };
    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING + 64.0)),
            Ab(nalgebra_glm::Vec2::new(220.0, 16.0)),
            Anchor::TopLeft,
        )
        .with_text(delta, 11.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_color::<UiBase>(delta_color)
        .without_pointer_events();

    tree.pop_parent();
}

fn showcase_quick_actions(tree: &mut UiTreeBuilder) {
    let theme = tree.active_theme().clone();
    let row_height = 88.0;
    let inner = showcase_card_inner(tree, row_height);
    tree.push_parent(inner);

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(0.0, 4.0)),
            Ab(nalgebra_glm::Vec2::new(180.0, 16.0)),
            Anchor::TopLeft,
        )
        .with_text("Quick actions", 13.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events();

    let row = tree
        .add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(0.0, 28.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 36.0)),
            Anchor::TopLeft,
        )
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            12.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(row);
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
    tree.pop_parent();

    tree.pop_parent();
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
        .flow_child(Rl(nalgebra_glm::Vec2::new(0.0, 100.0)))
        .flex_grow(1.0)
        .with_rect(8.0, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_state_offset::<UiHover>(nalgebra_glm::Vec2::new(0.0, -1.0))
        .with_state_offset::<UiPressed>(nalgebra_glm::Vec2::new(0.0, 1.0))
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.push_parent(chip);
    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(12.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(16.0, 16.0)),
            Anchor::CenterLeft,
        )
        .with_icon(icon, 14.0)
        .with_color::<UiBase>(icon_color)
        .without_pointer_events()
        .done();
    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(34.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(160.0, 16.0)),
            Anchor::CenterLeft,
        )
        .with_text(label, 12.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_color::<UiBase>(text_color)
        .without_pointer_events()
        .done();
    tree.pop_parent();
}

fn showcase_split_section(tree: &mut UiTreeBuilder) {
    let row = tree
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 280.0)),
        )
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            16.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(row);
    showcase_activity_card(tree);
    showcase_settings_card(tree);
    tree.pop_parent();
}

fn showcase_activity_card(tree: &mut UiTreeBuilder) {
    let theme = tree.active_theme().clone();
    let card = tree
        .add_node()
        .flow_child(Rl(nalgebra_glm::Vec2::new(0.0, 100.0)))
        .flex_grow(1.6)
        .with_rect(BLOCK_RADIUS, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_theme_effect_role(ThemeEffect::PanelEffect)
        .entity();
    tree.push_parent(card);
    let inner = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING)),
            Ab(nalgebra_glm::Vec2::new(-BLOCK_PADDING, -BLOCK_PADDING))
                + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .flow_with_alignment(
            FlowDirection::Vertical,
            0.0,
            10.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(inner);

    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 22.0)),
        )
        .with_text("Recent activity", 14.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events()
        .done();

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
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 28.0)),
            )
            .flow_with_alignment(
                FlowDirection::Horizontal,
                0.0,
                10.0,
                FlowAlignment::Start,
                FlowAlignment::Center,
            )
            .without_pointer_events()
            .entity();
        tree.push_parent(row);

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(8.0, 8.0)))
            .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(accent)
            .without_pointer_events()
            .done();

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(60.0, 18.0)))
            .with_text(who, 12.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .without_pointer_events()
            .done();

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, 18.0)))
            .flex_grow(1.0)
            .with_text(message, 12.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
            .without_pointer_events()
            .done();

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(60.0, 18.0)))
            .with_text(when, 11.0)
            .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
            .without_pointer_events()
            .done();

        tree.pop_parent();
    }

    tree.pop_parent();
    tree.pop_parent();
}

fn showcase_settings_card(tree: &mut UiTreeBuilder) {
    let theme = tree.active_theme().clone();
    let card = tree
        .add_node()
        .flow_child(Rl(nalgebra_glm::Vec2::new(0.0, 100.0)))
        .flex_grow(1.0)
        .with_rect(BLOCK_RADIUS, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_theme_effect_role(ThemeEffect::PanelEffect)
        .entity();
    tree.push_parent(card);
    let inner = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(BLOCK_PADDING, BLOCK_PADDING)),
            Ab(nalgebra_glm::Vec2::new(-BLOCK_PADDING, -BLOCK_PADDING))
                + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .flow_with_alignment(
            FlowDirection::Vertical,
            0.0,
            14.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .without_pointer_events()
        .entity();
    tree.push_parent(inner);

    tree.add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 22.0)),
        )
        .with_text("Workspace", 14.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events()
        .done();

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

    let _ = theme;
    tree.pop_parent();
    tree.pop_parent();
}

fn showcase_setting_row(
    tree: &mut UiTreeBuilder,
    label: &str,
    body: impl FnOnce(&mut UiTreeBuilder),
) {
    let row = tree
        .add_node()
        .flow_child(
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
        )
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            12.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .entity();
    tree.push_parent(row);
    tree.add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(96.0, 18.0)))
        .with_text(label, 12.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
        .without_pointer_events()
        .done();
    let widget_holder = tree
        .add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, 32.0)))
        .flex_grow(1.0)
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            8.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .entity();
    tree.push_parent(widget_holder);
    body(tree);
    tree.pop_parent();
    tree.pop_parent();
}

fn showcase_primary_button(tree: &mut UiTreeBuilder, label: &str, width: f32) -> Entity {
    let theme = tree.active_theme().clone();
    let entity = tree
        .add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(width, 36.0)))
        .with_rect(8.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(theme.accent_color)
        .with_color::<UiHover>(theme.accent_color_hovered)
        .with_color::<UiPressed>(theme.accent_color_active)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_theme_shadow_role::<UiBase>(ThemeShadow::ButtonShadow)
        .with_theme_shadow_role::<UiHover>(ThemeShadow::ButtonShadowHover)
        .with_state_offset::<UiHover>(nalgebra_glm::Vec2::new(0.0, -1.0))
        .with_state_offset::<UiPressed>(nalgebra_glm::Vec2::new(0.0, 1.0))
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.push_parent(entity);
    tree.add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(width - 16.0, 22.0)),
            Anchor::Center,
        )
        .with_text(label, 13.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0))
        .without_pointer_events()
        .done();
    tree.pop_parent();
    entity
}

fn showcase_secondary_button(tree: &mut UiTreeBuilder, label: &str, width: f32) -> Entity {
    let theme = tree.active_theme().clone();
    let entity = tree
        .add_node()
        .flow_child(Ab(nalgebra_glm::Vec2::new(width, 36.0)))
        .with_rect(8.0, 1.0, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_theme_effect_role(ThemeEffect::ButtonEffect)
        .with_state_offset::<UiHover>(nalgebra_glm::Vec2::new(0.0, -1.0))
        .with_state_offset::<UiPressed>(nalgebra_glm::Vec2::new(0.0, 1.0))
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.push_parent(entity);
    tree.add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
            Ab(nalgebra_glm::Vec2::new(width - 16.0, 22.0)),
            Anchor::Center,
        )
        .with_text(label, 13.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_theme_color::<UiBase>(ThemeColor::Text)
        .without_pointer_events()
        .done();
    tree.pop_parent();
    entity
}
