use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::data::state::{UiBase, UiStateTrait};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;

fn component<'a, T: Clone + Send + Sync + Default + 'static>(
    context: &'a crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &'a nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<&'a T> {
    context
        .commands
        .staged::<T>(entity)
        .or_else(|| storage.get::<T>(entity))
}

fn edit_component<T: Clone + Send + Sync + Default + 'static>(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    change: impl FnOnce(&mut T) + Send + Sync + 'static,
) {
    context.commands.queue(move |storage, _| {
        if let Some(value) = storage.get_mut::<T>(entity) {
            change(value);
        }
    });
}

fn node_visible(
    context: &crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> bool {
    component::<crate::components::UiLayoutNode>(context, storage, entity)
        .is_some_and(|node| node.visible)
}
pub fn ui_mark_layout_dirty(dirty: &mut crate::resources::RetainedUiDirty) {
    dirty.layout_dirty = true;
}

pub fn ui_mark_render_dirty(dirty: &mut crate::resources::RetainedUiDirty) {
    dirty.render_dirty = true;
}

pub fn ui_mark_children_dirty(dirty: &mut crate::resources::RetainedUiDirty) {
    dirty.layout_dirty = true;
}

pub fn ui_show_toast(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    message: &str,
    severity: crate::resources::ToastSeverity,
    duration: f32,
) {
    use crate::commands::builder::UiTreeBuilder;
    use crate::components::{UiAnimationPhase, UiAnimationType};
    use crate::resources::{ToastEntry, ToastSeverity};

    let corner = context.overlays.toast_corner;
    let container = if let Some(entity) = context.overlays.toast_container {
        entity
    } else {
        use crate::components::layout_types::{FlowAlignment, FlowDirection};
        use crate::data::units::UiValue;
        use crate::resources::ToastCorner;
        const TOAST_WIDTH: f32 = 320.0;
        const TOAST_MARGIN: f32 = 16.0;
        const TOP_OFFSET: f32 = 60.0;
        let (point_1, point_2, main_alignment): (UiValue<Vec2>, UiValue<Vec2>, FlowAlignment) =
            match corner {
                ToastCorner::TopLeft => (
                    Rl(Vec2::new(0.0, 0.0)) + Ab(Vec2::new(TOAST_MARGIN, TOP_OFFSET)),
                    Rl(Vec2::new(0.0, 100.0))
                        + Ab(Vec2::new(TOAST_MARGIN + TOAST_WIDTH, -TOAST_MARGIN)),
                    FlowAlignment::Start,
                ),
                ToastCorner::TopRight => (
                    Rl(Vec2::new(100.0, 0.0))
                        + Ab(Vec2::new(-(TOAST_MARGIN + TOAST_WIDTH), TOP_OFFSET)),
                    Rl(Vec2::new(100.0, 100.0)) + Ab(Vec2::new(-TOAST_MARGIN, -TOAST_MARGIN)),
                    FlowAlignment::Start,
                ),
                ToastCorner::BottomLeft => (
                    Rl(Vec2::new(0.0, 0.0)) + Ab(Vec2::new(TOAST_MARGIN, TOP_OFFSET)),
                    Rl(Vec2::new(0.0, 100.0))
                        + Ab(Vec2::new(TOAST_MARGIN + TOAST_WIDTH, -TOAST_MARGIN)),
                    FlowAlignment::End,
                ),
                ToastCorner::BottomRight => (
                    Rl(Vec2::new(100.0, 0.0))
                        + Ab(Vec2::new(-(TOAST_MARGIN + TOAST_WIDTH), TOP_OFFSET)),
                    Rl(Vec2::new(100.0, 100.0)) + Ab(Vec2::new(-TOAST_MARGIN, -TOAST_MARGIN)),
                    FlowAlignment::End,
                ),
            };
        let mut tree = UiTreeBuilder::new(context.reborrow());
        let container = tree
            .add_node()
            .boundary(point_1, point_2)
            .flow_with_alignment(
                FlowDirection::Vertical,
                0.0,
                8.0,
                main_alignment,
                FlowAlignment::Start,
            )
            .with_depth(UiDepthMode::Set(50.0))
            .entity();
        tree.finish();
        context.overlays.toast_container = Some(container);
        container
    };

    let accent = match severity {
        ToastSeverity::Info => Vec4::new(0.3, 0.6, 1.0, 1.0),
        ToastSeverity::Success => Vec4::new(0.2, 0.8, 0.3, 1.0),
        ToastSeverity::Warning => Vec4::new(1.0, 0.7, 0.1, 1.0),
        ToastSeverity::Error => Vec4::new(1.0, 0.25, 0.25, 1.0),
    };

    let text_slot = context.text_state.cache.add_text(message);
    let spawn_time = context.timing.current_time;

    let mut tree = UiTreeBuilder::from_parent(context.reborrow(), container);

    let intro_kind = match corner {
        crate::resources::ToastCorner::TopLeft | crate::resources::ToastCorner::BottomLeft => {
            UiAnimationType::SlideLeft
        }
        crate::resources::ToastCorner::TopRight | crate::resources::ToastCorner::BottomRight => {
            UiAnimationType::SlideRight
        }
    };
    let toast = tree
        .add_node()
        .flow_child(Ab(Vec2::new(320.0, 44.0)))
        .with_rect(6.0, 1.0, Vec4::new(0.15, 0.15, 0.22, 0.5))
        .color_raw::<UiBase>(Vec4::new(0.06, 0.06, 0.1, 0.95))
        .with_intro(intro_kind, 0.2)
        .with_outro(UiAnimationType::Fade, 0.15)
        .entity();

    tree.push_parent(toast);

    tree.add_node()
        .boundary(
            Rl(Vec2::new(0.0, 0.0)),
            Ab(Vec2::new(4.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)),
        )
        .with_rect(6.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(accent);

    tree.add_node()
        .window(
            Ab(Vec2::new(14.0, 18.0)),
            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(-20.0, 20.0)),
            Anchor::CenterLeft,
        )
        .with_text_slot(text_slot, 13.0)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .color_raw::<UiBase>(Vec4::new(0.9, 0.9, 0.95, 1.0));

    tree.pop_parent();
    tree.finish_subtree();

    context.edit::<crate::components::UiLayoutNode>(toast, |node| {
        if let Some(animation) = &mut node.animation {
            animation.phase = UiAnimationPhase::IntroPlaying;
            animation.progress = 0.0;
        }
        node.visible = true;
    });

    ui_mark_layout_dirty(context.dirty);
    ui_mark_render_dirty(context.dirty);

    context.overlays.toast_entries.push(ToastEntry {
        entity: toast,
        spawn_time,
        duration,
        dismissing: false,
    });
}

pub fn ui_tick_toasts(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
) {
    let current_time = context.timing.current_time;
    let entries = std::mem::take(&mut context.overlays.toast_entries);
    let mut kept = Vec::new();
    let mut to_despawn = Vec::new();

    for mut entry in entries {
        let elapsed = (current_time - entry.spawn_time) as f32;

        if entry.dismissing {
            let phase =
                component::<crate::components::UiLayoutNode>(context, storage, entry.entity)
                    .and_then(|node| node.animation.as_ref())
                    .map(|anim| anim.phase);

            if phase == Some(crate::components::UiAnimationPhase::OutroComplete)
                || !node_visible(context, storage, entry.entity)
            {
                to_despawn.push(entry.entity);
            } else {
                kept.push(entry);
            }
        } else if elapsed >= entry.duration {
            entry.dismissing = true;
            ui_set_visible(context.dirty, context.commands, entry.entity, false);
            kept.push(entry);
        } else {
            kept.push(entry);
        }
    }

    context.overlays.toast_entries = kept;

    for entity in to_despawn {
        ui_despawn_node(context, entity);
    }
}

pub fn ui_set_selected(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    selected: bool,
) {
    let accent = context.theme.active_theme().accent_color;
    let selected_bg = Vec4::new(accent.x, accent.y, accent.z, 0.3);

    context.commands.queue(move |storage, _| {
        if let Some(data) = storage.get_mut::<crate::components::UiSelectableLabelData>(entity) {
            data.selected = selected;
        }
        if let Some(color) = storage.get_mut::<crate::components::UiNodeColor>(entity) {
            color.colors[crate::data::state::UiSelected::INDEX] =
                if selected { Some(selected_bg) } else { None };
        }
        if let Some(weights) = storage.get_mut::<crate::components::UiStateWeights>(entity) {
            weights.weights[crate::data::state::UiSelected::INDEX] =
                if selected { 1.0 } else { 0.0 };
        }
    });
}

pub fn ui_tree_node_set_expanded(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    expanded: bool,
) {
    let update = component::<crate::components::UiTreeNodeData>(context, storage, entity)
        .map(|data| (data.arrow_text_slot, data.children_container));
    apply_tree_expansion(context, update, entity, expanded);
}

pub fn ui_tree_node_expand_on_build(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    expanded: bool,
) {
    let update = context
        .commands
        .staged::<crate::components::UiTreeNodeData>(entity)
        .map(|data| (data.arrow_text_slot, data.children_container));
    apply_tree_expansion(context, update, entity, expanded);
}

fn apply_tree_expansion(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    update: Option<(usize, nightshade::ecs::Entity)>,
    entity: nightshade::ecs::Entity,
    expanded: bool,
) {
    let Some((arrow_slot, children)) = update else {
        return;
    };
    context
        .text_state
        .cache
        .set_text(arrow_slot, if expanded { "\u{25BC}" } else { "\u{25B6}" });
    edit_component::<crate::components::UiLayoutNode>(context, children, move |node| {
        node.visible = expanded;
    });
    edit_component::<crate::components::UiTreeNodeData>(context, entity, move |data| {
        data.expanded = expanded;
    });
}

pub fn ui_show_modal(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) {
    let backdrop = component::<crate::components::UiModalDialogData>(context, storage, entity)
        .map(|data| data.backdrop_entity);

    if let Some(backdrop_entity) = backdrop {
        edit_component::<crate::components::UiLayoutNode>(context, backdrop_entity, |node| {
            node.visible = true;
        });
        edit_component::<crate::components::UiLayoutNode>(context, entity, |node| {
            node.visible = true;
        });
        edit_component::<crate::components::UiModalDialogData>(context, entity, |data| {
            data.result = None;
        });
        context.overlays.active_modal = Some(entity);
        ui_mark_layout_dirty(context.dirty);
    }
}

pub fn ui_widget_content(
    context: &crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<nightshade::ecs::Entity> {
    if let Some(data) =
        component::<crate::components::UiCollapsingHeaderData>(context, storage, entity)
    {
        return Some(data.content_entity);
    }
    if let Some(data) = component::<crate::components::UiScrollAreaData>(context, storage, entity) {
        return Some(data.content_entity);
    }
    if let Some(data) = component::<crate::components::UiPanelData>(context, storage, entity) {
        return Some(data.content_entity);
    }
    if let Some(data) = component::<crate::components::UiModalDialogData>(context, storage, entity)
    {
        return Some(data.content_entity);
    }
    if let Some(data) = component::<crate::components::UiTreeViewData>(context, storage, entity) {
        return Some(data.content_entity);
    }
    if let Some(data) =
        component::<crate::components::UiTileContainerData>(context, storage, entity)
    {
        return Some(data.container_entity);
    }
    Some(entity)
}

pub fn ui_show_command_palette(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) {
    let (backdrop, text_input) = if let Some(data) =
        component::<crate::components::UiCommandPaletteData>(context, storage, entity)
    {
        (data.backdrop_entity, data.text_input_entity)
    } else {
        return;
    };

    edit_component::<crate::components::UiLayoutNode>(context, backdrop, |node| {
        node.visible = true;
    });
    edit_component::<crate::components::UiLayoutNode>(context, entity, |node| {
        node.visible = true;
    });

    ui_text_input_set_value(context.commands, text_input, "");
    context.interaction.focused_entity = Some(text_input);

    edit_component::<crate::components::UiCommandPaletteData>(context, entity, |data| {
        data.open = true;
        data.executed_command = None;
        data.filter_text.clear();
        data.selected_index = 0;
        data.filtered_indices = (0..data.commands.len()).collect();
    });

    ui_command_palette_rebuild_results(context, storage, entity);
    ui_mark_layout_dirty(context.dirty);
}

pub fn ui_open_context_menu(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    menu: nightshade::ecs::Entity,
    screen_position: nalgebra_glm::Vec2,
) {
    let Some(popup) = component::<crate::components::UiContextMenuData>(context, storage, menu)
        .map(|data| data.popup_entity)
    else {
        return;
    };

    edit_component::<crate::components::UiLayoutNode>(context, popup, move |node| {
        if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
            node.base_layout.as_mut()
        {
            window.position = crate::data::units::Ab(screen_position).into();
        }
        node.visible = true;
    });

    edit_component::<crate::components::UiContextMenuData>(context, menu, |data| {
        data.open = true;
        data.clicked_item = None;
    });

    context.overlays.active_context_menu = Some(menu);
    ui_mark_layout_dirty(context.dirty);
}

pub fn ui_command_palette_register(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    label: &str,
    shortcut: &str,
    category: &str,
) {
    let entry = CommandEntry {
        label: label.to_string(),
        shortcut: shortcut.to_string(),
        category: category.to_string(),
        enabled: true,
    };
    context.edit::<crate::components::UiCommandPaletteData>(entity, move |data| {
        data.commands.push(entry);
        data.filtered_indices = (0..data.commands.len()).collect();
    });
}

pub fn ui_command_palette_rebuild_results(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) {
    let data = if let Some(data) =
        component::<crate::components::UiCommandPaletteData>(context, storage, entity)
    {
        data.clone()
    } else {
        return;
    };

    for (pool_index, &row_entity) in data.result_entities.iter().enumerate() {
        if pool_index < data.filtered_indices.len() {
            let cmd_index = data.filtered_indices[pool_index];
            let cmd = &data.commands[cmd_index];
            let label_slot = data.result_text_slots[pool_index * 2];
            let shortcut_slot = data.result_text_slots[pool_index * 2 + 1];

            let display = if cmd.category.is_empty() {
                cmd.label.clone()
            } else {
                format!("{}: {}", cmd.category, cmd.label)
            };
            context.text_state.cache.set_text(label_slot, &display);
            context
                .text_state
                .cache
                .set_text(shortcut_slot, &cmd.shortcut);

            edit_component::<crate::components::UiLayoutNode>(context, row_entity, |node| {
                node.visible = true;
            });
        } else {
            edit_component::<crate::components::UiLayoutNode>(context, row_entity, |node| {
                node.visible = false;
            });
        }
    }
}
