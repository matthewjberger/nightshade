use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layout_types::FlowDirection;
use crate::components::*;
use crate::data::state::{UiBase, UiFocused, UiHover, UiPressed};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_button(&mut self, text: &str) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let button_height = theme.button_height;
        let corner_radius = theme.corner_radius;
        let border_width = theme.border_width;
        let border_color = theme.border_color;

        let text_slot = self.text_state_mut().cache.add_text(text);

        let button_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .with_rect(corner_radius, border_width, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_theme_effect_role(crate::components::ThemeEffect::ButtonEffect)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::ButtonShadow)
            .with_theme_shadow_role::<UiHover>(crate::components::ThemeShadow::ButtonShadowHover)
            .with_theme_shadow_role::<UiPressed>(
                crate::components::ThemeShadow::ButtonShadowPressed,
            )
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_transition::<UiPressed>(12.0, 8.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_state_offset::<UiHover>(Vec2::new(0.0, -2.0))
            .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .with_text_slot(text_slot, font_size)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        self.commands().set(
            button_entity,
            UiButtonData {
                clicked: false,
                text_slot: Some(text_slot),
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(button_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Button);
        });
        self.assign_tab_index(button_entity);

        button_entity
    }

    pub fn add_icon_button(
        &mut self,
        icon: crate::data::icons::IconGlyph,
        font_size: f32,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let button_height = theme.button_height;
        let corner_radius = theme.corner_radius;
        let border_width = theme.border_width;
        let border_color = theme.border_color;

        let button_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .with_rect(corner_radius, border_width, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_theme_effect_role(crate::components::ThemeEffect::ButtonEffect)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::ButtonShadow)
            .with_theme_shadow_role::<UiHover>(crate::components::ThemeShadow::ButtonShadowHover)
            .with_theme_shadow_role::<UiPressed>(
                crate::components::ThemeShadow::ButtonShadowPressed,
            )
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_transition::<UiPressed>(12.0, 8.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_state_offset::<UiHover>(Vec2::new(0.0, -2.0))
            .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .absolute_fill(0.0)
                    .with_icon(icon, font_size)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        self.commands().set(
            button_entity,
            UiButtonData {
                clicked: false,
                text_slot: None,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(button_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Button);
        });
        self.assign_tab_index(button_entity);

        button_entity
    }

    pub fn add_button_colored(&mut self, text: &str, color: Vec4) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let button_height = theme.button_height;
        let corner_radius = theme.corner_radius;
        let border_width = theme.border_width;
        let border_color = theme.border_color;

        let hover_color = Vec4::new(
            (color.x + 0.1).min(1.0),
            (color.y + 0.1).min(1.0),
            (color.z + 0.1).min(1.0),
            color.w,
        );
        let active_color = Vec4::new(
            (color.x - 0.1).max(0.0),
            (color.y - 0.1).max(0.0),
            (color.z - 0.1).max(0.0),
            color.w,
        );

        let text_slot = self.text_state_mut().cache.add_text(text);

        let button_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .with_rect(corner_radius, border_width, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .color_raw::<UiBase>(color)
            .color_raw::<UiHover>(hover_color)
            .color_raw::<UiPressed>(active_color)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::ButtonShadow)
            .with_theme_shadow_role::<UiHover>(crate::components::ThemeShadow::ButtonShadowHover)
            .with_theme_shadow_role::<UiPressed>(
                crate::components::ThemeShadow::ButtonShadowPressed,
            )
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_transition::<UiPressed>(12.0, 8.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_state_offset::<UiHover>(Vec2::new(0.0, -2.0))
            .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .with_text_slot(text_slot, font_size)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        self.commands().set(
            button_entity,
            UiButtonData {
                clicked: false,
                text_slot: Some(text_slot),
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(button_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Button);
        });
        self.assign_tab_index(button_entity);

        button_entity
    }

    pub fn add_spinner(&mut self) -> nightshade::ecs::Entity {
        let dot_count = 8;
        let size = 28.0_f32;
        let radius = 9.0_f32;
        let dot_size = 5.0_f32;
        let center = size * 0.5;

        let entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(size, size)))
            .entity();

        self.push_parent(entity);

        let theme = self.active_theme();
        let accent = theme.accent_color;

        let mut dots = Vec::with_capacity(dot_count);
        for index in 0..dot_count {
            let angle = (index as f32 / dot_count as f32) * std::f32::consts::TAU
                - std::f32::consts::FRAC_PI_2;
            let x = center + angle.cos() * radius - dot_size * 0.5;
            let y = center + angle.sin() * radius - dot_size * 0.5;
            let dot = self
                .add_node()
                .window(
                    Ab(Vec2::new(x, y)),
                    Ab(Vec2::new(dot_size, dot_size)),
                    Anchor::TopLeft,
                )
                .with_rect(dot_size * 0.5, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(accent)
                .entity();
            dots.push(dot);
        }

        self.pop_parent();

        self.commands().set(
            entity,
            UiSpinnerData {
                dots,
                rotation_speed: 6.5,
                phase: 0.0,
            },
        );

        entity
    }

    pub fn add_button_rich(&mut self, spans: &[TextSpan]) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let button_height = theme.button_height;
        let corner_radius = theme.corner_radius;
        let border_width = theme.border_width;
        let border_color = theme.border_color;
        let text_color = theme.text_color;

        let button_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .with_rect(corner_radius, border_width, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_theme_effect_role(crate::components::ThemeEffect::ButtonEffect)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::ButtonShadow)
            .with_theme_shadow_role::<UiHover>(crate::components::ThemeShadow::ButtonShadowHover)
            .with_theme_shadow_role::<UiPressed>(
                crate::components::ThemeShadow::ButtonShadowPressed,
            )
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_transition::<UiPressed>(12.0, 8.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_state_offset::<UiHover>(Vec2::new(0.0, -2.0))
            .with_state_offset::<UiPressed>(Vec2::new(0.0, 2.0))
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        self.push_parent(button_entity);

        let span_padding = 8.0;
        let span_height = (button_height - span_padding * 2.0).max(font_size);

        let span_container = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .flow(FlowDirection::Horizontal, span_padding, 0.0)
            .align_main(crate::components::layout_types::FlowAlignment::Center)
            .align_cross(crate::components::layout_types::FlowAlignment::Center)
            .entity();

        self.push_parent(span_container);

        for span in spans {
            self.build_text_span(span, font_size, text_color, Some(span_height), false);
        }

        self.pop_parent();
        self.pop_parent();

        self.commands().set(
            button_entity,
            UiButtonData {
                clicked: false,
                text_slot: None,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(button_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Button);
        });
        self.assign_tab_index(button_entity);

        button_entity
    }

    pub fn add_selectable_label(
        &mut self,
        text: &str,
        group_id: Option<u32>,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;

        let text_slot = self.text_state_mut().cache.add_text(text);

        let label_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiPressed>(14.0, 8.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .boundary(Ab(Vec2::new(8.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .with_text_slot(text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        self.commands().set(
            label_entity,
            UiSelectableLabelData {
                selected: false,
                changed: false,
                text_slot,
                group_id,
            },
        );
        if let Some(gid) = group_id {
            self.groups_mut()
                .selectable_labels
                .entry(gid)
                .or_default()
                .push(label_entity);
        }

        label_entity
    }
}
