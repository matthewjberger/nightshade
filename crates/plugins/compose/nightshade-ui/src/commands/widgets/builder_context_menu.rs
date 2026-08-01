use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layer::UiLayer;
use crate::components::layout_types::{FlowAlignment, FlowDirection};
use crate::components::*;
use crate::data::state::{UiBase, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use super::{ContextMenuBuilder, ContextMenuBuilderEntry, ContextMenuTheme};

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_context_menu(&mut self, items: &[(&str, Option<&str>)]) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let button_height = theme.button_height;

        let mut item_entities = Vec::new();
        let context_items: Vec<ContextMenuItem> = items
            .iter()
            .map(|(label, shortcut)| ContextMenuItem {
                label: label.to_string(),
                shortcut: shortcut.map(|text| text.to_string()),
                separator: label.is_empty(),
            })
            .collect();

        let popup_height = items
            .iter()
            .fold(0.0f32, |acc, (label, _)| {
                if label.is_empty() {
                    acc + 5.0
                } else {
                    acc + button_height
                }
            })
            .min(crate::MAX_POPUP_HEIGHT);

        let popup_width = {
            let font_engine = &mut self.text_state_mut().font_engine;
            let (content_max, has_shortcut) = measure_menu_content_width(
                font_engine,
                items
                    .iter()
                    .filter(|(label, _)| !label.is_empty())
                    .map(|(label, shortcut)| (*label, shortcut.unwrap_or(""))),
                font_size,
            );
            let padding = 16.0 + if has_shortcut { 16.0 } else { 0.0 };
            (content_max + padding).max(200.0).ceil()
        };

        let popup_entity = self
            .add_node()
            .window(
                Ab(Vec2::new(0.0, 0.0)),
                Ab(Vec2::new(popup_width, popup_height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(35.0))
            .with_visible(false)
            .flow(FlowDirection::Vertical, 2.0, 0.0)
            .entity();

        self.push_parent(popup_entity);

        for (label, shortcut) in items {
            if label.is_empty() {
                self.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 1.0)))
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Border)
                    .entity();
                item_entities.push(None);
                continue;
            }

            let label_slot = self.text_state_mut().cache.add_text(*label);

            let item_entity = if let Some(shortcut_text) = shortcut {
                let shortcut_slot = self.text_state_mut().cache.add_text(*shortcut_text);
                self.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                    .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Background)
                    .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                    .with_interaction()
                    .with_transition::<UiHover>(8.0, 6.0)
                    .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                    .flow(FlowDirection::Horizontal, 8.0, 0.0)
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
                            .auto_size_padding(Vec2::new(4.0, 0.0))
                            .with_text_slot(shortcut_slot, font_size * 0.85)
                            .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                            .entity();
                    })
                    .entity()
            } else {
                self.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                    .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Background)
                    .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                    .with_interaction()
                    .with_transition::<UiHover>(8.0, 6.0)
                    .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(Ab(Vec2::new(8.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                            .with_text_slot(label_slot, font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::Text)
                            .entity();
                    })
                    .entity()
            };

            self.edit::<crate::components::UiNodeInteraction>(item_entity, |interaction| {
                interaction.accessible_role = Some(AccessibleRole::MenuItem);
            });
            item_entities.push(Some(item_entity));
        }

        self.pop_parent();

        let menu_entity = popup_entity;
        self.commands()
            .set(menu_entity, UiNodeInteraction::default());
        self.edit::<crate::components::UiLayoutNode>(menu_entity, |node| {
            node.pointer_events = true;
        });
        self.edit::<crate::components::UiNodeInteraction>(menu_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Menu);
        });
        self.commands().set(
            menu_entity,
            UiContextMenuData {
                items: context_items,
                open: false,
                clicked_item: None,
                popup_entity,
                item_entities,
                item_defs: Vec::new(),
            },
        );

        menu_entity
    }

    pub fn add_context_menu_from_builder(
        &mut self,
        builder: ContextMenuBuilder,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let button_height = theme.button_height;

        let mut command_counter = 0usize;
        let context_menu_theme = ContextMenuTheme {
            font_size,
            corner_radius,
            button_height,
        };

        let popup_entity = self.build_context_menu_popup(
            &builder.entries,
            &mut command_counter,
            &context_menu_theme,
            0,
        );

        let item_defs = if let Some(data) =
            self.component::<crate::components::UiContextMenuData>(popup_entity)
        {
            data.item_defs.clone()
        } else {
            Vec::new()
        };

        let item_entities: Vec<Option<nightshade::ecs::Entity>> =
            item_defs.iter().map(|def| def.row_entity).collect();

        self.commands()
            .set(popup_entity, UiNodeInteraction::default());
        self.edit::<crate::components::UiLayoutNode>(popup_entity, |node| {
            node.pointer_events = true;
        });
        self.edit::<crate::components::UiNodeInteraction>(popup_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Menu);
        });
        self.commands().set(
            popup_entity,
            UiContextMenuData {
                items: Vec::new(),
                open: false,
                clicked_item: None,
                popup_entity,
                item_entities,
                item_defs,
            },
        );

        popup_entity
    }

    fn build_context_menu_popup(
        &mut self,
        entries: &[ContextMenuBuilderEntry],
        command_counter: &mut usize,
        theme: &ContextMenuTheme,
        depth_level: u32,
    ) -> nightshade::ecs::Entity {
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let button_height = theme.button_height;

        let border_color = self.active_theme().border_color;

        let popup_height = entries
            .iter()
            .fold(0.0f32, |acc, entry| match entry {
                ContextMenuBuilderEntry::Separator => acc + 5.0,
                _ => acc + button_height,
            })
            .min(crate::MAX_POPUP_HEIGHT);

        let popup_width = compute_context_menu_width(self, entries, font_size);

        let popup_entity = self
            .add_node()
            .window(
                Ab(Vec2::new(0.0, 0.0)),
                Ab(Vec2::new(popup_width, popup_height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(36.0 + depth_level as f32))
            .with_visible(false)
            .flow(FlowDirection::Vertical, 2.0, 0.0)
            .entity();

        self.push_parent(popup_entity);

        let mut item_defs = Vec::new();

        for entry in entries {
            match entry {
                ContextMenuBuilderEntry::Separator => {
                    self.add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 1.0)))
                        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .with_theme_color::<UiBase>(ThemeColor::Border)
                        .entity();
                    item_defs.push(ContextMenuItemDef {
                        label: String::new(),
                        shortcut: String::new(),
                        kind: ContextMenuItemKind::Separator,
                        row_entity: None,
                        command_id: None,
                        tag: 0,
                        binding: None,
                    });
                }
                ContextMenuBuilderEntry::Item {
                    label,
                    shortcut,
                    tag,
                } => {
                    let cmd_id = *command_counter;
                    *command_counter += 1;

                    let label_slot = self.text_state_mut().cache.add_text(label);
                    let row_entity = if shortcut.is_empty() {
                        self.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)),
                            )
                            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                            .with_theme_color::<UiBase>(ThemeColor::Background)
                            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                            .with_interaction()
                            .with_transition::<UiHover>(8.0, 6.0)
                            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                            .with_children(|tree| {
                                tree.add_node()
                                    .boundary(Ab(Vec2::new(8.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                                    .with_text_slot(label_slot, font_size)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_theme_color::<UiBase>(ThemeColor::Text)
                                    .entity();
                            })
                            .entity()
                    } else {
                        let shortcut_slot = self.text_state_mut().cache.add_text(shortcut);
                        self.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)),
                            )
                            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                            .with_theme_color::<UiBase>(ThemeColor::Background)
                            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                            .with_interaction()
                            .with_transition::<UiHover>(8.0, 6.0)
                            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                            .flow(FlowDirection::Horizontal, 8.0, 0.0)
                            .with_children(|tree| {
                                tree.add_node()
                                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                    .flex_grow(1.0)
                                    .with_text_slot(label_slot, font_size)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_theme_color::<UiBase>(ThemeColor::Text)
                                    .entity();
                                tree.add_node()
                                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                    .auto_size(crate::components::AutoSizeMode::Width)
                                    .auto_size_padding(Vec2::new(4.0, 0.0))
                                    .with_text_slot(shortcut_slot, font_size * 0.85)
                                    .with_text_alignment(
                                        TextAlignment::Right,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                                    .entity();
                            })
                            .entity()
                    };

                    item_defs.push(ContextMenuItemDef {
                        label: label.clone(),
                        shortcut: shortcut.clone(),
                        kind: ContextMenuItemKind::Action,
                        row_entity: Some(row_entity),
                        command_id: Some(cmd_id),
                        tag: *tag,
                        binding: ShortcutBinding::parse(shortcut),
                    });
                }
                ContextMenuBuilderEntry::Checkable {
                    label,
                    shortcut,
                    checked,
                    tag,
                } => {
                    let cmd_id = *command_counter;
                    *command_counter += 1;

                    let label_slot = self.text_state_mut().cache.add_text(label);
                    let check_text_slot = self.text_state_mut().cache.add_text(if *checked {
                        "\u{2713}"
                    } else {
                        ""
                    });

                    let row_entity = self
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                        .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .with_theme_color::<UiBase>(ThemeColor::Background)
                        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                        .with_interaction()
                        .with_transition::<UiHover>(8.0, 6.0)
                        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                        .flow(FlowDirection::Horizontal, 8.0, 0.0)
                        .with_children(|tree| {
                            tree.add_node()
                                .flow_child(Ab(Vec2::new(16.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                                .with_text_slot(check_text_slot, font_size)
                                .with_text_alignment(
                                    TextAlignment::Center,
                                    VerticalAlignment::Middle,
                                )
                                .with_theme_color::<UiBase>(ThemeColor::Accent)
                                .entity();
                            tree.add_node()
                                .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                .flex_grow(1.0)
                                .with_text_slot(label_slot, font_size)
                                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                                .with_theme_color::<UiBase>(ThemeColor::Text)
                                .entity();
                            if !shortcut.is_empty() {
                                let shortcut_slot = tree.text_state_mut().cache.add_text(shortcut);
                                tree.add_node()
                                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                    .auto_size(crate::components::AutoSizeMode::Width)
                                    .auto_size_padding(Vec2::new(4.0, 0.0))
                                    .with_text_slot(shortcut_slot, font_size * 0.85)
                                    .with_text_alignment(
                                        TextAlignment::Right,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                                    .entity();
                            }
                        })
                        .entity();

                    item_defs.push(ContextMenuItemDef {
                        label: label.clone(),
                        shortcut: shortcut.clone(),
                        kind: ContextMenuItemKind::Checkable {
                            checked: *checked,
                            check_text_slot,
                        },
                        row_entity: Some(row_entity),
                        command_id: Some(cmd_id),
                        tag: *tag,
                        binding: ShortcutBinding::parse(shortcut),
                    });
                }
                ContextMenuBuilderEntry::Submenu { label, children } => {
                    let label_slot = self.text_state_mut().cache.add_text(label);
                    let arrow_slot = self.text_state_mut().cache.add_text("\u{25B6}");

                    let row_entity = self
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                        .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .with_theme_color::<UiBase>(ThemeColor::Background)
                        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                        .with_interaction()
                        .with_transition::<UiHover>(8.0, 6.0)
                        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                        .flow(FlowDirection::Horizontal, 8.0, 0.0)
                        .with_children(|tree| {
                            tree.add_node()
                                .flow_child(Rl(Vec2::new(0.0, 100.0)))
                                .flex_grow(1.0)
                                .with_text_slot(label_slot, font_size)
                                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                                .with_theme_color::<UiBase>(ThemeColor::Text)
                                .entity();
                            tree.add_node()
                                .flow_child(Ab(Vec2::new(16.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                                .with_text_slot(arrow_slot, font_size * 0.85)
                                .with_text_alignment(
                                    TextAlignment::Center,
                                    VerticalAlignment::Middle,
                                )
                                .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                                .entity();
                        })
                        .entity();

                    self.pop_parent();
                    let child_popup = self.build_context_menu_popup(
                        children,
                        command_counter,
                        theme,
                        depth_level + 1,
                    );
                    self.push_parent(popup_entity);

                    let child_defs = if let Some(data) =
                        self.component::<crate::components::UiContextMenuData>(child_popup)
                    {
                        data.item_defs.clone()
                    } else {
                        Vec::new()
                    };

                    item_defs.push(ContextMenuItemDef {
                        label: label.clone(),
                        shortcut: String::new(),
                        kind: ContextMenuItemKind::Submenu {
                            children: child_defs,
                            popup_entity: child_popup,
                            open: false,
                        },
                        row_entity: Some(row_entity),
                        command_id: None,
                        tag: 0,
                        binding: None,
                    });
                }
                ContextMenuBuilderEntry::WidgetRow { label } => {
                    let cmd_id = *command_counter;
                    *command_counter += 1;

                    let label_slot = self.text_state_mut().cache.add_text(label);

                    let row_entity = self
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                        .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .with_theme_color::<UiBase>(ThemeColor::Background)
                        .with_interaction()
                        .flow_with_alignment(
                            FlowDirection::Horizontal,
                            8.0,
                            0.0,
                            FlowAlignment::Start,
                            FlowAlignment::Center,
                        )
                        .entity();

                    let content_entity = self.in_parent(row_entity, |tree| {
                        tree.add_node()
                            .flow_child(Ab(Vec2::new(0.0, button_height)))
                            .flex_grow(1.0)
                            .with_text_slot(label_slot, font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::Text)
                            .entity();
                        tree.add_node()
                            .flow_child(Ab(Vec2::new(0.0, button_height)))
                            .flow_with_alignment(
                                FlowDirection::Horizontal,
                                0.0,
                                4.0,
                                FlowAlignment::End,
                                FlowAlignment::Center,
                            )
                            .auto_size(crate::components::AutoSizeMode::Width)
                            .entity()
                    });

                    item_defs.push(ContextMenuItemDef {
                        label: label.clone(),
                        shortcut: String::new(),
                        kind: ContextMenuItemKind::Widget { content_entity },
                        row_entity: Some(row_entity),
                        command_id: Some(cmd_id),
                        tag: 0,
                        binding: None,
                    });
                }
            }
        }

        self.pop_parent();

        self.commands().set(
            popup_entity,
            UiContextMenuData {
                items: Vec::new(),
                open: false,
                clicked_item: None,
                popup_entity,
                item_entities: item_defs
                    .iter()
                    .map(|definition| definition.row_entity)
                    .collect(),
                item_defs,
            },
        );

        popup_entity
    }
}

fn measure_menu_content_width<'b>(
    font_engine: &mut nightshade_text::contract::data::font_engine::FontEngine,
    rows: impl Iterator<Item = (&'b str, &'b str)>,
    font_size: f32,
) -> (f32, bool) {
    let mut content_max = 0.0f32;
    let mut has_shortcut = false;
    for (label, shortcut) in rows {
        let label_width = crate::data::units::measure_text_width(font_engine, label, font_size);
        let shortcut_width = if shortcut.is_empty() {
            0.0
        } else {
            has_shortcut = true;
            crate::data::units::measure_text_width(font_engine, shortcut, font_size * 0.85)
        };
        content_max = content_max.max(label_width + shortcut_width);
    }
    (content_max, has_shortcut)
}

fn compute_context_menu_width(
    tree: &mut UiTreeBuilder<'_, '_>,
    entries: &[ContextMenuBuilderEntry],
    font_size: f32,
) -> f32 {
    let mut has_checkable = false;
    let mut has_submenu = false;
    let mut rows: Vec<(&str, &str)> = Vec::with_capacity(entries.len());
    for entry in entries {
        let row = match entry {
            ContextMenuBuilderEntry::Item {
                label, shortcut, ..
            } => (label.as_str(), shortcut.as_str()),
            ContextMenuBuilderEntry::Checkable {
                label, shortcut, ..
            } => {
                has_checkable = true;
                (label.as_str(), shortcut.as_str())
            }
            ContextMenuBuilderEntry::Submenu { label, .. } => {
                has_submenu = true;
                (label.as_str(), "")
            }
            ContextMenuBuilderEntry::WidgetRow { label } => (label.as_str(), ""),
            ContextMenuBuilderEntry::Separator => continue,
        };
        rows.push(row);
    }

    let font_engine = &mut tree.text_state_mut().font_engine;
    let (content_max, has_shortcut) =
        measure_menu_content_width(font_engine, rows.iter().copied(), font_size);

    let mut padding = 16.0;
    if has_checkable {
        padding += 24.0;
    }
    if has_shortcut {
        padding += 16.0;
    }
    if has_submenu {
        padding += 16.0;
    }
    (content_max + padding).max(200.0).ceil()
}
