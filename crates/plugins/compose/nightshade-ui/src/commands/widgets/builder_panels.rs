use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layer::UiLayer;
use crate::components::layout_types::FlowDirection;
use crate::components::*;
use crate::data::state::{UiBase, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::{Anchor, Rect};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_floating_panel(
        &mut self,
        id: &str,
        title: &str,
        rect: Rect,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;

        let title_slot = self.text_state_mut().cache.add_text(title);
        let collapse_slot = self.text_state_mut().cache.add_text("\u{00D7}");

        let focus_order = self.runtime_mut().next_focus_order;
        self.runtime_mut().next_focus_order += 1;

        let panel_entity = self
            .add_node()
            .window(
                Ab(Vec2::new(rect.min.x, rect.min.y)),
                Ab(Vec2::new(rect.width(), rect.height())),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_effect_role(crate::components::ThemeEffect::PanelEffect)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::PanelShadow)
            .with_layer(UiLayer::FloatingPanels)
            .with_clip()
            .with_depth(crate::components::UiDepthMode::Set(
                20.0 + focus_order as f32 * 10.0,
            ))
            .entity();

        let (header_entity, collapse_button_entity, content_entity) =
            self.in_parent(panel_entity, |tree| {
                let header_entity = tree
                    .add_node()
                    .boundary(
                        Rl(Vec2::new(0.0, 0.0)),
                        Ab(Vec2::new(0.0, 32.0)) + Rl(Vec2::new(100.0, 0.0)),
                    )
                    .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::BackgroundActive)
                    .with_clip()
                    .with_interaction()
                    .with_cursor_icon(nightshade_platform::CursorIcon::Grab)
                    .entity();

                let collapse_button_entity = tree.in_parent(header_entity, |tree| {
                    tree.add_node()
                        .window(
                            Ab(Vec2::new(8.0, 16.0)),
                            Ab(Vec2::new(200.0, 20.0)),
                            Anchor::CenterLeft,
                        )
                        .with_text_slot(title_slot, font_size * 0.85)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_theme_color::<UiBase>(ThemeColor::Text)
                        .entity();

                    tree.add_node()
                        .window(
                            Rl(Vec2::new(100.0, 50.0)) + Ab(Vec2::new(-8.0, 0.0)),
                            Ab(Vec2::new(20.0, 20.0)),
                            Anchor::CenterRight,
                        )
                        .with_rect(2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .with_theme_color::<UiBase>(ThemeColor::BackgroundHover)
                        .with_theme_color::<UiHover>(ThemeColor::Accent)
                        .with_interaction()
                        .with_transition::<UiHover>(8.0, 6.0)
                        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                        .with_children(|tree| {
                            tree.add_node()
                                .window(
                                    Rl(Vec2::new(50.0, 50.0)),
                                    Ab(Vec2::new(20.0, 20.0)),
                                    Anchor::Center,
                                )
                                .with_text_slot(collapse_slot, font_size * 0.8)
                                .with_text_alignment(
                                    TextAlignment::Center,
                                    VerticalAlignment::Middle,
                                )
                                .with_theme_color::<UiBase>(ThemeColor::Text)
                                .entity();
                        })
                        .entity()
                });

                let content_entity = tree
                    .add_node()
                    .boundary(Ab(Vec2::new(0.0, 32.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 8.0, 4.0)
                    .with_clip()
                    .entity();

                (header_entity, collapse_button_entity, content_entity)
            });

        self.commands().set(
            panel_entity,
            UiPanelData {
                id: id.to_string(),
                title: title.to_string(),
                title_text_slot: title_slot,
                content_entity,
                header_entity,
                collapsed: false,
                panel_kind: UiPanelKind::Floating,
                focus_order,
                pinned: false,
                min_size: Vec2::new(150.0, 100.0),
                drag_offset: None,
                resize_edge: None,
                resize_start_rect: None,
                resize_start_mouse: None,
                undocked_rect: None,
                default_dock_size: 300.0,
                collapse_button_entity: Some(collapse_button_entity),
                collapse_button_text_slot: Some(collapse_slot),
                header_visible: true,
                resizable: true,
            },
        );

        panel_entity
    }

    fn build_docked_panel(
        &mut self,
        id: &str,
        title: &str,
        panel_kind: UiPanelKind,
        default_size: f32,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;

        let title_slot = self.text_state_mut().cache.add_text(title);

        let focus_order = self.runtime_mut().next_focus_order;
        self.runtime_mut().next_focus_order += 1;

        let panel_entity = self
            .add_node()
            .window(
                Ab(Vec2::new(0.0, 0.0)),
                Ab(Vec2::new(default_size, default_size)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_effect_role(crate::components::ThemeEffect::PanelEffect)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::PanelShadow)
            .with_layer(UiLayer::DockedPanels)
            .with_clip()
            .with_depth(crate::components::UiDepthMode::Set(
                10.0 + focus_order as f32 * 10.0,
            ))
            .entity();

        let (header_entity, content_entity) = self.in_parent(panel_entity, |tree| {
            let header_entity = tree
                .add_node()
                .boundary(
                    Rl(Vec2::new(0.0, 0.0)),
                    Ab(Vec2::new(0.0, 32.0)) + Rl(Vec2::new(100.0, 0.0)),
                )
                .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::BackgroundActive)
                .with_clip()
                .with_interaction()
                .with_cursor_icon(nightshade_platform::CursorIcon::Grab)
                .with_children(|tree| {
                    tree.add_node()
                        .window(
                            Ab(Vec2::new(8.0, 16.0)),
                            Ab(Vec2::new(200.0, 20.0)),
                            Anchor::CenterLeft,
                        )
                        .with_text_slot(title_slot, font_size * 0.85)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_theme_color::<UiBase>(ThemeColor::Text)
                        .entity();
                })
                .entity();

            let content_entity = tree
                .add_node()
                .boundary(Ab(Vec2::new(0.0, 32.0)), Rl(Vec2::new(100.0, 100.0)))
                .flow(FlowDirection::Vertical, 8.0, 4.0)
                .with_clip()
                .entity();

            (header_entity, content_entity)
        });

        self.commands().set(
            panel_entity,
            UiPanelData {
                id: id.to_string(),
                title: title.to_string(),
                title_text_slot: title_slot,
                content_entity,
                header_entity,
                collapsed: false,
                panel_kind,
                focus_order,
                pinned: false,
                min_size: Vec2::new(100.0, 60.0),
                drag_offset: None,
                resize_edge: None,
                resize_start_rect: None,
                resize_start_mouse: None,
                undocked_rect: None,
                default_dock_size: default_size,
                collapse_button_entity: None,
                collapse_button_text_slot: None,
                header_visible: true,
                resizable: true,
            },
        );

        panel_entity
    }

    pub fn add_docked_panel_top(
        &mut self,
        id: &str,
        title: &str,
        default_height: f32,
    ) -> nightshade::ecs::Entity {
        self.build_docked_panel(id, title, UiPanelKind::DockedTop, default_height)
    }

    pub fn add_docked_panel_bottom(
        &mut self,
        id: &str,
        title: &str,
        default_height: f32,
    ) -> nightshade::ecs::Entity {
        self.build_docked_panel(id, title, UiPanelKind::DockedBottom, default_height)
    }

    pub fn add_docked_panel_left(
        &mut self,
        id: &str,
        title: &str,
        default_width: f32,
    ) -> nightshade::ecs::Entity {
        self.build_docked_panel(id, title, UiPanelKind::DockedLeft, default_width)
    }

    pub fn add_docked_panel_right(
        &mut self,
        id: &str,
        title: &str,
        default_width: f32,
    ) -> nightshade::ecs::Entity {
        self.build_docked_panel(id, title, UiPanelKind::DockedRight, default_width)
    }

    pub fn add_color_picker(&mut self, initial_color: Vec4) -> nightshade::ecs::Entity {
        self.add_color_picker_with_mode(initial_color, crate::components::ColorPickerMode::Rgb)
    }

    pub fn add_color_picker_with_mode(
        &mut self,
        initial_color: Vec4,
        mode: crate::components::ColorPickerMode,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;

        let root_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flow(FlowDirection::Horizontal, 4.0, 8.0)
            .auto_height()
            .entity();

        self.push_parent(root_entity);

        let swatch_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(48.0, 48.0)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .color_raw::<UiBase>(initial_color)
            .with_interaction()
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let sliders_container = self
            .add_node()
            .flow_child(Ab(Vec2::new(0.0, 0.0)))
            .flex_grow(1.0)
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .entity();

        self.push_parent(sliders_container);

        let slider_entities = match mode {
            crate::components::ColorPickerMode::Rgb => {
                let slider_r = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, initial_color.x)
                        .precision(2)
                        .prefix("R: "),
                );
                let slider_g = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, initial_color.y)
                        .precision(2)
                        .prefix("G: "),
                );
                let slider_b = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, initial_color.z)
                        .precision(2)
                        .prefix("B: "),
                );
                let slider_a = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, initial_color.w)
                        .precision(2)
                        .prefix("A: "),
                );
                [slider_r, slider_g, slider_b, slider_a]
            }
            crate::components::ColorPickerMode::Hsv => {
                let hsv = crate::data::color::Hsva::from_rgba(initial_color);
                let slider_h = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 360.0, hsv.hue)
                        .precision(0)
                        .prefix("H: ")
                        .suffix("\u{00b0}"),
                );
                let slider_s = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, hsv.saturation)
                        .precision(2)
                        .prefix("S: "),
                );
                let slider_v = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, hsv.value)
                        .precision(2)
                        .prefix("V: "),
                );
                let slider_a = self.add_slider_configured(
                    crate::components::SliderConfig::new(0.0, 1.0, initial_color.w)
                        .precision(2)
                        .prefix("A: "),
                );
                [slider_h, slider_s, slider_v, slider_a]
            }
        };

        self.pop_parent();
        self.pop_parent();

        let popup_width = 280.0;
        let popup_height = 280.0;
        let popup_entity = self
            .add_node()
            .window(
                Ab(Vec2::new(0.0, 56.0)),
                Ab(Vec2::new(popup_width, popup_height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::PanelShadow)
            .with_layer(UiLayer::Popups)
            .with_depth(crate::components::UiDepthMode::Set(35.0))
            .with_visible(false)
            .with_interaction()
            .with_clip()
            .entity();
        self.edit::<crate::components::UiLayoutNode>(popup_entity, |node| {
            node.flow_layout = Some(crate::components::layout_types::FlowLayout {
                direction: FlowDirection::Vertical,
                padding: 12.0,
                spacing: 8.0,
                alignment: crate::components::layout_types::FlowAlignment::Start,
                cross_alignment: crate::components::layout_types::FlowAlignment::Start,
                wrap: false,
            });
        });
        self.push_parent(popup_entity);
        let popup_wheel_entity = self.add_color_wheel(initial_color);
        self.pop_parent();

        self.commands().set(
            root_entity,
            UiColorPickerData {
                color: initial_color,
                changed: false,
                swatch_entity,
                slider_entities,
                mode,
                popup_entity,
                popup_wheel_entity,
                popup_open: false,
            },
        );

        root_entity
    }

    pub fn add_color_wheel(&mut self, initial_color: Vec4) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;

        let hsv = crate::data::color::Hsva::from_rgba(initial_color);
        let hue_normalized = (hsv.hue / 360.0).clamp(0.0, 1.0);
        let saturation = hsv.saturation.clamp(0.0, 1.0);
        let value = hsv.value.clamp(0.0, 1.0);
        let alpha = initial_color.w.clamp(0.0, 1.0);

        let area_size = 180.0;
        let dot_size = 12.0;

        let root_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flow(FlowDirection::Horizontal, 4.0, 12.0)
            .auto_height()
            .entity();

        self.push_parent(root_entity);

        let wheel_holder = self
            .add_node()
            .flow_child(Ab(Vec2::new(area_size, area_size)))
            .entity();
        self.push_parent(wheel_holder);

        let wheel_entity = self
            .add_node()
            .window(
                Rl(Vec2::new(0.0, 0.0)),
                Rl(Vec2::new(100.0, 100.0)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
            .with_effect(crate::resources::theme::RectEffect::ColorWheel { value })
            .with_interaction()
            .with_cursor_icon(nightshade_platform::CursorIcon::Crosshair)
            .entity();

        let (dot_x, dot_y) = crate::data::color::color_wheel_dot_offset(
            area_size, area_size, dot_size, hsv.hue, saturation,
        );

        let dot_entity = self
            .add_node()
            .window(
                Ab(Vec2::new(dot_x, dot_y)),
                Ab(Vec2::new(dot_size, dot_size)),
                Anchor::TopLeft,
            )
            .with_rect(dot_size * 0.5, 2.0, Vec4::new(0.0, 0.0, 0.0, 1.0))
            .color_raw::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
            .entity();

        self.pop_parent();

        let sliders_container = self
            .add_node()
            .flow_child(Ab(Vec2::new(0.0, 0.0)))
            .flex_grow(1.0)
            .flow(FlowDirection::Vertical, 0.0, 6.0)
            .entity();
        self.push_parent(sliders_container);

        let swatch_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 32.0)))
            .with_rect(corner_radius, 1.0, border_color)
            .color_raw::<UiBase>(initial_color)
            .entity();

        let value_slider_entity = self.add_slider_configured(
            crate::components::SliderConfig::new(0.0, 1.0, value)
                .precision(2)
                .prefix("V: "),
        );
        let alpha_slider_entity = self.add_slider_configured(
            crate::components::SliderConfig::new(0.0, 1.0, alpha)
                .precision(2)
                .prefix("A: "),
        );

        self.pop_parent();
        self.pop_parent();

        self.commands().set(
            root_entity,
            crate::components::UiColorWheelData {
                color: initial_color,
                hue: hue_normalized,
                saturation,
                value,
                alpha,
                changed: false,
                wheel_entity,
                dot_entity,
                swatch_entity,
                value_slider_entity,
                alpha_slider_entity,
            },
        );

        root_entity
    }

    pub fn panel_content(&self, panel: nightshade::ecs::Entity) -> Option<nightshade::ecs::Entity> {
        self.component::<UiPanelData>(panel)
            .map(|data| data.content_entity)
    }
}
