use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layer::UiLayer;
use crate::components::layout_types::FlowDirection;
use crate::data::state::{UiBase, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;
impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_modal_dialog(
        &mut self,
        title: &str,
        width: f32,
        height: f32,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;

        let title_slot = self.text_state_mut().cache.add_text(title);

        let backdrop_entity = self
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.5))
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(40.0))
            .with_interaction()
            .with_visible(false)
            .entity();

        let dialog_entity = self
            .add_node()
            .window(
                Rl(Vec2::new(50.0, 50.0)) + Ab(Vec2::new(-width / 2.0, -height / 2.0)),
                Ab(Vec2::new(width, height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_effect_role(crate::components::ThemeEffect::PanelEffect)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::ModalShadow)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(41.0))
            .with_visible(false)
            .entity();

        let content_entity = self.in_parent(dialog_entity, |tree| {
            tree.add_node()
                .boundary(
                    Rl(Vec2::new(0.0, 0.0)),
                    Ab(Vec2::new(0.0, 36.0)) + Rl(Vec2::new(100.0, 0.0)),
                )
                .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::BackgroundActive)
                .with_children(|tree| {
                    tree.add_node()
                        .window(
                            Ab(Vec2::new(12.0, 18.0)),
                            Ab(Vec2::new(200.0, 20.0)),
                            Anchor::CenterLeft,
                        )
                        .with_text_slot(title_slot, font_size)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_theme_color::<UiBase>(ThemeColor::Accent)
                        .entity();
                })
                .entity();

            tree.add_node()
                .boundary(Ab(Vec2::new(0.0, 36.0)), Rl(Vec2::new(100.0, 100.0)))
                .flow(FlowDirection::Vertical, 12.0, 8.0)
                .entity()
        });

        self.commands().set(
            dialog_entity,
            UiModalDialogData {
                title_text_slot: title_slot,
                content_entity,
                backdrop_entity,
                ok_button: None,
                cancel_button: None,
                result: None,
            },
        );

        dialog_entity
    }

    pub fn add_confirm_dialog(&mut self, title: &str, message: &str) -> nightshade::ecs::Entity {
        let dialog_entity = self.add_modal_dialog(title, 350.0, 180.0);

        let content = widget::<UiModalDialogData>(self.commands(), dialog_entity)
            .map(|data| data.content_entity)
            .unwrap_or(dialog_entity);

        let accent_color = self.active_theme().accent_color;

        let ok_button;
        let cancel_button;

        {
            let mut subtree = crate::commands::builder::UiTreeBuilder::from_parent(
                self.context_reborrow(),
                content,
            );

            subtree.add_label(message);

            subtree.add_spacing(8.0);

            let button_row = subtree
                .add_node()
                .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 36.0)))
                .flow(FlowDirection::Horizontal, 0.0, 8.0)
                .entity();

            subtree.push_parent(button_row);
            cancel_button = subtree.add_button("Cancel");
            ok_button = subtree.add_button_colored("OK", accent_color);
            subtree.pop_parent();

            subtree.edit::<crate::components::UiLayoutNode>(cancel_button, |node| {
                node.flow_child_size = Some(Rl(Vec2::new(50.0, 0.0)) + Ab(Vec2::new(-4.0, 36.0)));
            });
            subtree.edit::<crate::components::UiLayoutNode>(ok_button, |node| {
                node.flow_child_size = Some(Rl(Vec2::new(50.0, 0.0)) + Ab(Vec2::new(-4.0, 36.0)));
            });
            subtree.finish_subtree();
        }

        self.edit::<crate::components::UiModalDialogData>(dialog_entity, move |data| {
            data.ok_button = Some(ok_button);
            data.cancel_button = Some(cancel_button);
        });

        dialog_entity
    }

    pub fn add_command_palette(&mut self, pool_size: usize) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let accent_color = theme.accent_color;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let row_height = font_size * 1.8;

        let backdrop_entity = self
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::zeros())
            .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.3))
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(42.0))
            .with_interaction()
            .with_visible(false)
            .entity();

        let dialog_width = 500.0f32;
        let dialog_height = (pool_size as f32 * row_height) + row_height + 8.0;

        let dialog_entity = self
            .add_node()
            .window(
                Rl(Vec2::new(50.0, 0.0)) + Ab(Vec2::new(-dialog_width / 2.0, 80.0)),
                Ab(Vec2::new(dialog_width, dialog_height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(43.0))
            .with_visible(false)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        let (text_input_entity, scroll_entity, result_entities, result_text_slots) = self
            .in_parent(dialog_entity, |tree| {
                let mut result_entities = Vec::with_capacity(pool_size);
                let mut result_text_slots = Vec::with_capacity(pool_size);

                let text_input_entity = tree.add_text_input("Type a command...");
                tree.edit::<crate::components::UiLayoutNode>(text_input_entity, move |node| {
                    node.flow_child_size =
                        Some(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, row_height)));
                });

                let scroll_entity = tree.add_scroll_area_fill(0.0, 0.0);
                let content = widget::<UiScrollAreaData>(tree.commands(), scroll_entity)
                    .map(|data| data.content_entity)
                    .unwrap_or(scroll_entity);

                tree.push_parent(content);

                for _ in 0..pool_size {
                    let label_slot = tree.text_state_mut().cache.add_text("");
                    let shortcut_slot = tree.text_state_mut().cache.add_text("");

                    let row = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, row_height)))
                        .with_rect(2.0, 0.0, Vec4::zeros())
                        .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .color_raw::<UiHover>(Vec4::new(
                            accent_color.x,
                            accent_color.y,
                            accent_color.z,
                            0.2,
                        ))
                        .with_interaction()
                        .with_transition::<UiHover>(8.0, 6.0)
                        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                        .flow(FlowDirection::Horizontal, 8.0, 0.0)
                        .with_visible(false)
                        .with_children(|tree| {
                            tree.add_node()
                                .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                .flex_grow(1.0)
                                .with_text_slot(label_slot, font_size)
                                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                                .with_theme_color::<UiBase>(ThemeColor::Text)
                                .entity();
                            tree.add_node()
                                .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                .auto_size(crate::components::AutoSizeMode::Width)
                                .auto_size_padding(Vec2::new(8.0, 0.0))
                                .with_text_slot(shortcut_slot, font_size * 0.85)
                                .with_text_alignment(
                                    TextAlignment::Right,
                                    VerticalAlignment::Middle,
                                )
                                .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                                .entity();
                        })
                        .entity();

                    result_entities.push(row);
                    result_text_slots.push(label_slot);
                    result_text_slots.push(shortcut_slot);
                }

                tree.pop_parent();

                (
                    text_input_entity,
                    scroll_entity,
                    result_entities,
                    result_text_slots,
                )
            });

        let palette_entity = dialog_entity;
        self.commands()
            .set(palette_entity, UiNodeInteraction::default());
        self.edit::<crate::components::UiLayoutNode>(palette_entity, |node| {
            node.pointer_events = true;
        });
        self.edit::<crate::components::UiNodeInteraction>(palette_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Dialog);
        });
        self.commands().set(
            palette_entity,
            UiCommandPaletteData {
                commands: Vec::new(),
                filter_text: String::new(),
                filtered_indices: Vec::new(),
                selected_index: 0,
                open: false,
                executed_command: None,
                text_input_entity,
                result_entities,
                result_text_slots,
                backdrop_entity,
                dialog_entity,
                pool_size,
                scroll_entity,
            },
        );

        palette_entity
    }
}
