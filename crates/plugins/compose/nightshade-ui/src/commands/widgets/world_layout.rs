use nalgebra_glm;

use crate::data::state::{UiBase, UiStateTrait};

use crate::prelude::*;
use nightshade::ecs::system_param::Commands;
pub fn ui_set_clipboard_text(
    runtime: &mut crate::resources::RetainedUiRuntime,
    text: impl Into<String>,
) {
    let text = text.into();
    runtime.clipboard_text = text.clone();
    nightshade_platform::commands::host::write_clipboard(&text);
}

pub fn ui_read_system_clipboard(runtime: &mut crate::resources::RetainedUiRuntime) -> String {
    if let Some(text) = nightshade_platform::commands::host::read_clipboard() {
        runtime.clipboard_text = text.clone();
        text
    } else {
        runtime.clipboard_text.clone()
    }
}

pub fn ui_events(frame: &crate::resources::RetainedUiFrame) -> &[crate::resources::UiEvent] {
    &frame.events
}

pub fn ui_button_clicks(
    frame: &crate::resources::RetainedUiFrame,
) -> impl Iterator<Item = nightshade::ecs::Entity> + '_ {
    ui_events(frame).iter().filter_map(|event| match event {
        crate::resources::UiEvent::ButtonClicked(entity) => Some(*entity),
        _ => None,
    })
}

pub fn ui_despawn_node(
    context: &mut crate::commands::builder::UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
) {
    context.dirty.layout_dirty = true;
    context.commands.queue(move |storage, resources| {
        resources
            .res_mut::<nightshade::ecs::HierarchyIndex>()
            .sync(storage);

        let mut stack = vec![entity];
        let mut all_entities = Vec::new();
        while let Some(current) = stack.pop() {
            all_entities.push(current);
            for child in resources
                .res::<nightshade::ecs::HierarchyIndex>()
                .children(current)
            {
                stack.push(*child);
            }
        }

        for descendant in &all_entities {
            if let Some(content) = storage.get::<crate::components::UiNodeContent>(*descendant)
                && let crate::components::UiNodeContent::Text { text_slot, .. } = content
            {
                let slot = *text_slot;
                resources
                    .res_mut::<nightshade_text::contract::resources::TextState>()
                    .cache
                    .remove_text(slot);
            }

            if let Some(data) = storage
                .get::<crate::components::UiRadioData>(*descendant)
                .cloned()
                && let Some(group) = resources
                    .res_mut::<crate::resources::RetainedUiGroups>()
                    .radio
                    .get_mut(&data.group_id)
            {
                group.retain(|entity| *entity != *descendant);
            }
            if let Some(data) = storage
                .get::<crate::components::UiSelectableLabelData>(*descendant)
                .cloned()
                && let Some(group_id) = data.group_id
                && let Some(group) = resources
                    .res_mut::<crate::resources::RetainedUiGroups>()
                    .selectable_labels
                    .get_mut(&group_id)
            {
                group.retain(|entity| *entity != *descendant);
            }

            let mut slots: Vec<usize> = Vec::new();
            if let Some(data) = storage.get::<crate::components::UiButtonData>(*descendant)
                && let Some(text_slot) = data.text_slot
            {
                slots.push(text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiSliderData>(*descendant) {
                slots.push(data.text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiTextInputData>(*descendant) {
                slots.push(data.text_slot);
            }
            if let Some(data) =
                storage.get::<crate::components::UiCollapsingHeaderData>(*descendant)
            {
                slots.push(data.arrow_text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiDropdownData>(*descendant) {
                slots.push(data.header_text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiMenuData>(*descendant) {
                slots.push(data.label_text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiPanelData>(*descendant) {
                slots.push(data.title_text_slot);
                if let Some(slot) = data.collapse_button_text_slot {
                    slots.push(slot);
                }
            }
            if let Some(data) = storage.get::<crate::components::UiSelectableLabelData>(*descendant)
            {
                slots.push(data.text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiDragValueData>(*descendant) {
                slots.push(data.text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiTreeNodeData>(*descendant) {
                slots.push(data.text_slot);
                slots.push(data.arrow_text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiModalDialogData>(*descendant) {
                slots.push(data.title_text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiTabBarData>(*descendant) {
                slots.extend_from_slice(&data.tab_text_slots);
            }
            if let Some(data) = storage.get::<crate::components::UiTextAreaData>(*descendant) {
                slots.push(data.text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiRichTextData>(*descendant) {
                slots.extend_from_slice(&data.span_text_slots);
            }
            if let Some(data) = storage.get::<crate::components::UiDataGridData>(*descendant) {
                slots.extend_from_slice(&data.header_text_slots);
                for row in &data.pool_rows {
                    slots.extend_from_slice(&row.cell_text_slots);
                }
            }
            if let Some(data) = storage.get::<crate::components::UiCommandPaletteData>(*descendant)
            {
                slots.extend_from_slice(&data.result_text_slots);
            }
            if let Some(data) = storage.get::<crate::components::UiRichTextEditorData>(*descendant)
            {
                slots.push(data.text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiBreadcrumbData>(*descendant) {
                slots.extend_from_slice(&data.segment_text_slots);
            }
            if let Some(data) = storage.get::<crate::components::UiMultiSelectData>(*descendant) {
                slots.push(data.header_text_slot);
            }
            if let Some(data) = storage.get::<crate::components::UiDatePickerData>(*descendant) {
                slots.push(data.header_text_slot);
                slots.push(data.month_label_slot);
                slots.extend_from_slice(&data.day_text_slots);
            }
            for slot in slots {
                resources
                    .res_mut::<nightshade_text::contract::resources::TextState>()
                    .cache
                    .remove_text(slot);
            }
        }

        let despawned: std::collections::HashSet<nightshade::ecs::Entity> =
            all_entities.iter().copied().collect();
        resources
            .res_mut::<crate::resources::RetainedUiAccessibility>()
            .named_entities
            .retain(|_, stored| !despawned.contains(stored));
        resources
            .res_mut::<crate::resources::RetainedUiAccessibility>()
            .test_id_map
            .retain(|_, stored| !despawned.contains(stored));

        clear_despawned_singletons(resources, &despawned);

        storage.despawn_recursive(entity);
        ui_mark_children_dirty(resources.res_mut::<crate::resources::RetainedUiDirty>());
    });
}

fn clear_despawned_singletons(
    resources: &mut nightshade::ecs::ResourceMap,
    despawned: &std::collections::HashSet<nightshade::ecs::Entity>,
) {
    let interaction = resources.res_mut::<crate::resources::RetainedUiInteraction>();
    if interaction
        .focused_entity
        .is_some_and(|entity| despawned.contains(&entity))
    {
        interaction.focused_entity = None;
    }
    if interaction
        .hovered_entity
        .is_some_and(|entity| despawned.contains(&entity))
    {
        interaction.hovered_entity = None;
    }
    if interaction
        .active_entity
        .is_some_and(|entity| despawned.contains(&entity))
    {
        interaction.active_entity = None;
    }
    if interaction
        .last_click
        .is_some_and(|(entity, _, _)| despawned.contains(&entity))
    {
        interaction.last_click = None;
    }

    let overlays = &mut resources.res_mut::<crate::resources::RetainedUiOverlays>();
    overlays
        .popup_entities
        .retain(|entity| !despawned.contains(entity));
    if overlays
        .active_context_menu
        .is_some_and(|entity| despawned.contains(&entity))
    {
        overlays.active_context_menu = None;
    }
    if overlays
        .active_modal
        .is_some_and(|entity| despawned.contains(&entity))
    {
        overlays.active_modal = None;
    }
    if overlays
        .toast_container
        .is_some_and(|entity| despawned.contains(&entity))
    {
        overlays.toast_container = None;
    }
    if overlays
        .dock_indicator_panel
        .is_some_and(|entity| despawned.contains(&entity))
    {
        overlays.dock_indicator_panel = None;
        overlays.dock_indicator_active = false;
    }
    if let Some(tooltip) = &overlays.tooltip_state
        && (despawned.contains(&tooltip.entity)
            || tooltip
                .tooltip_entity
                .is_some_and(|entity| despawned.contains(&entity)))
    {
        overlays.tooltip_state = None;
    }

    let drag = &mut resources.res_mut::<crate::resources::RetainedUiDragDrop>();
    if drag
        .active
        .as_ref()
        .is_some_and(|active| despawned.contains(&active.source))
    {
        drag.active = None;
    }
    if drag
        .over_entity
        .is_some_and(|entity| despawned.contains(&entity))
    {
        drag.over_entity = None;
    }
}

pub fn ui_focus(
    interaction: &mut crate::resources::RetainedUiInteraction,
    entity: nightshade::ecs::Entity,
) {
    interaction.focused_entity = Some(entity);
}

pub fn ui_set_disabled(commands: &mut Commands, entity: nightshade::ecs::Entity, disabled: bool) {
    commands.queue(move |storage, _| {
        if let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(entity) {
            interaction.disabled = disabled;
        }
        let base_color = storage
            .get::<crate::components::UiNodeColor>(entity)
            .and_then(|color| color.colors[UiBase::INDEX]);
        if let Some(color_component) = storage.get_mut::<crate::components::UiNodeColor>(entity) {
            color_component.colors[crate::data::state::UiDisabled::INDEX] =
                match (disabled, base_color) {
                    (true, Some(base)) => Some(nalgebra_glm::Vec4::new(
                        base.x,
                        base.y,
                        base.z,
                        base.w * 0.4,
                    )),
                    (true, None) => color_component.colors[crate::data::state::UiDisabled::INDEX],
                    (false, _) => None,
                };
        }
    });
}

pub fn ui_upload_image(
    images: &mut crate::resources::UiImageLayerAllocator,
    runtime: &mut crate::resources::RetainedUiRuntime,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Option<UiImageUpload> {
    if width == 0 || height == 0 {
        return None;
    }
    let layer = images.allocate()?;
    let upload = ui_repaint_image(runtime, layer, rgba, width, height);
    if upload.is_none() {
        images.release(layer);
    }
    upload
}

pub fn ui_repaint_image(
    runtime: &mut crate::resources::RetainedUiRuntime,
    layer: u32,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Option<UiImageUpload> {
    use nalgebra_glm::Vec2;
    use nightshade_paint::resources::TEXTURE_LAYER_SIZE;

    if width == 0 || height == 0 {
        return None;
    }

    let target = TEXTURE_LAYER_SIZE;
    let scale_factor = (target as f32 / width as f32).min(target as f32 / height as f32);
    let scaled_width = ((width as f32 * scale_factor).round() as u32).clamp(1, target);
    let scaled_height = ((height as f32 * scale_factor).round() as u32).clamp(1, target);

    let scaled_pixels = if scaled_width == width && scaled_height == height {
        rgba.to_vec()
    } else {
        let source: image::RgbaImage = image::ImageBuffer::from_raw(width, height, rgba.to_vec())?;
        image::imageops::resize(
            &source,
            scaled_width,
            scaled_height,
            image::imageops::FilterType::Triangle,
        )
        .into_raw()
    };

    let mut padded = vec![0u8; (target * target * 4) as usize];
    let row_bytes = (scaled_width * 4) as usize;
    let target_row_bytes = (target * 4) as usize;
    for row in 0..scaled_height as usize {
        let source_offset = row * row_bytes;
        let target_offset = row * target_row_bytes;
        padded[target_offset..target_offset + row_bytes]
            .copy_from_slice(&scaled_pixels[source_offset..source_offset + row_bytes]);
    }

    runtime
        .pending_image_uploads
        .push(crate::resources::UiImageLayerUpload {
            layer,
            rgba_data: padded,
            width: target,
            height: target,
        });

    let uv_max = Vec2::new(
        scaled_width as f32 / target as f32,
        scaled_height as f32 / target as f32,
    );
    Some(UiImageUpload {
        layer,
        uv_min: Vec2::new(0.0, 0.0),
        uv_max,
        content_width: scaled_width,
        content_height: scaled_height,
    })
}

pub fn ui_release_image_layer(allocator: &mut crate::resources::UiImageLayerAllocator, layer: u32) {
    allocator.release(layer);
}

pub fn ui_set_named(
    accessibility: &mut crate::resources::RetainedUiAccessibility,
    entity: nightshade::ecs::Entity,
    name: &str,
) {
    accessibility
        .named_entities
        .insert(name.to_string(), entity);
}

pub fn ui_named_entity(
    accessibility: &crate::resources::RetainedUiAccessibility,
    name: &str,
) -> Option<nightshade::ecs::Entity> {
    accessibility.named_entities.get(name).copied()
}

pub fn ui_set_test_id(
    accessibility: &mut crate::resources::RetainedUiAccessibility,
    commands: &mut Commands,
    entity: nightshade::ecs::Entity,
    test_id: &str,
) {
    accessibility
        .test_id_map
        .insert(test_id.to_string(), entity);
    let test_id = test_id.to_string();
    commands.queue(move |storage, _| {
        if storage
            .get::<crate::components::UiNodeInteraction>(entity)
            .is_none()
        {
            storage.set(entity, crate::components::UiNodeInteraction::default());
            if let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(entity) {
                node.pointer_events = true;
            }
        }
        if let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(entity) {
            interaction.test_id = Some(test_id);
        }
    });
}

pub fn ui_set_visible(
    dirty: &mut crate::resources::RetainedUiDirty,
    commands: &mut Commands,
    entity: impl Into<Option<nightshade::ecs::Entity>>,
    visible: bool,
) {
    let Some(entity) = entity.into() else {
        return;
    };
    ui_mark_layout_dirty(dirty);
    commands.queue(move |storage, _| {
        let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(entity) else {
            return;
        };
        let Some(animation) = &mut node.animation else {
            node.visible = visible;
            return;
        };
        if visible {
            if animation.intro.is_some() {
                animation.phase = crate::components::UiAnimationPhase::IntroPlaying;
                animation.progress = 0.0;
            }
            node.visible = true;
        } else if animation.outro.is_some() {
            animation.phase = crate::components::UiAnimationPhase::OutroPlaying;
            animation.progress = 0.0;
        } else {
            animation.phase = crate::components::UiAnimationPhase::Idle;
            node.visible = false;
        }
    });
}

pub fn ui_node_visible(nodes: &mut UiLayoutNodes<'_, '_>, entity: nightshade::ecs::Entity) -> bool {
    let mut visible = false;
    nodes.for_entity(entity, |(node,)| visible = node.visible);
    visible
}

pub fn ui_toggle_visible(
    nodes: &mut UiLayoutNodes<'_, '_>,
    dirty: &mut crate::resources::RetainedUiDirty,
    commands: &mut Commands,
    entity: nightshade::ecs::Entity,
) {
    let visible = ui_node_visible(nodes, entity);
    ui_set_visible(dirty, commands, entity, !visible);
}

pub type UiLayoutNodes<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut crate::components::UiLayoutNode,)>;

pub fn ui_node_effectively_visible(
    nodes: &mut UiLayoutNodes<'_, '_>,
    parents: &mut UiParents<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> bool {
    let mut visible = |entity| {
        let mut result = None;
        nodes.for_entity(entity, |(node,)| result = Some(node.visible));
        result
    };
    let mut parent = |entity| {
        let mut result = None;
        parents.for_entity(entity, |(child_of,)| result = Some(child_of.0));
        result
    };
    ui_chain_visible(&mut visible, &mut parent, entity)
}

pub fn ui_chain_visible(
    visible: &mut dyn FnMut(nightshade::ecs::Entity) -> Option<bool>,
    parent: &mut dyn FnMut(nightshade::ecs::Entity) -> Option<nightshade::ecs::Entity>,
    entity: nightshade::ecs::Entity,
) -> bool {
    if !visible(entity).unwrap_or(false) {
        return false;
    }
    let mut current = entity;
    loop {
        let Some(above) = parent(current) else {
            return true;
        };
        match visible(above) {
            Some(true) => current = above,
            Some(false) => return false,
            None => return true,
        }
    }
}

pub type UiParents<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item nightshade::ecs::ChildOf,)>;

pub fn ui_set_window_layout(
    dirty: &mut crate::resources::RetainedUiDirty,
    commands: &mut Commands,
    entity: impl Into<Option<nightshade::ecs::Entity>>,
    layout: crate::components::layout_types::WindowLayout,
) {
    let Some(entity) = entity.into() else {
        return;
    };
    ui_mark_layout_dirty(dirty);
    commands.queue(move |storage, _| {
        if let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(entity) {
            node.base_layout = Some(crate::components::layout_types::UiLayoutType::Window(
                layout,
            ));
        }
    });
}

pub fn ui_set_text_cached(
    commands: &mut Commands,
    entity: impl Into<Option<nightshade::ecs::Entity>>,
    last: &mut String,
    text: impl AsRef<str>,
) -> bool {
    let Some(entity) = entity.into() else {
        return false;
    };
    let text = text.as_ref();
    if last == text {
        return false;
    }
    last.clear();
    last.push_str(text);
    ui_set_text(commands, entity, text);
    true
}

pub fn ui_set_text(commands: &mut Commands, entity: nightshade::ecs::Entity, text: &str) {
    let text = text.to_string();
    commands.queue(move |storage, resources| {
        let Some(text_slot) = world_text_slot(storage, entity) else {
            return;
        };
        resources
            .res_mut::<nightshade_text::contract::resources::TextState>()
            .cache
            .set_text(text_slot, &text);
    });
}

fn world_text_slot(
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<usize> {
    if let Some(crate::components::UiNodeContent::Text { text_slot, .. }) =
        storage.get::<crate::components::UiNodeContent>(entity)
    {
        return Some(*text_slot);
    }
    if let Some(data) = storage.get::<crate::components::UiButtonData>(entity) {
        return data.text_slot;
    }
    if let Some(data) = storage.get::<crate::components::UiSliderData>(entity) {
        return Some(data.text_slot);
    }
    if let Some(data) = storage.get::<crate::components::UiTextInputData>(entity) {
        return Some(data.text_slot);
    }
    if let Some(data) = storage.get::<crate::components::UiSelectableLabelData>(entity) {
        return Some(data.text_slot);
    }
    if let Some(data) = storage.get::<crate::components::UiDragValueData>(entity) {
        return Some(data.text_slot);
    }
    None
}

pub fn ui_set_color<S: UiStateTrait>(
    commands: &mut Commands,
    entity: nightshade::ecs::Entity,
    color: nalgebra_glm::Vec4,
) {
    commands.queue(move |storage, _| {
        if let Some(node_color) = storage.get_mut::<crate::components::UiNodeColor>(entity) {
            node_color.colors[S::INDEX] = Some(color);
        }
        if let Some(binding) = storage.get_mut::<crate::components::UiThemeBinding>(entity) {
            binding.color_roles[S::INDEX] = None;
        }
    });
}

pub fn ui_text_slot_for_entity(
    contents: &mut UiTextSlots<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> Option<usize> {
    let mut slot = None;
    contents.for_entity(
        entity,
        |(content, button, slider, text_input, selectable, drag_value)| {
            if let Some(crate::components::UiNodeContent::Text { text_slot, .. }) = content {
                slot = Some(*text_slot);
            } else if let Some(data) = button {
                slot = data.text_slot;
            } else if let Some(data) = slider {
                slot = Some(data.text_slot);
            } else if let Some(data) = text_input {
                slot = Some(data.text_slot);
            } else if let Some(data) = selectable {
                slot = Some(data.text_slot);
            } else if let Some(data) = drag_value {
                slot = Some(data.text_slot);
            }
        },
    );
    slot
}

pub type UiTextSlots<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        Option<&'item crate::components::UiNodeContent>,
        Option<&'item crate::components::UiButtonData>,
        Option<&'item crate::components::UiSliderData>,
        Option<&'item crate::components::UiTextInputData>,
        Option<&'item crate::components::UiSelectableLabelData>,
        Option<&'item crate::components::UiDragValueData>,
    ),
>;

#[derive(Clone, Copy, Debug)]
pub struct UiImageUpload {
    pub layer: u32,
    pub uv_min: nalgebra_glm::Vec2,
    pub uv_max: nalgebra_glm::Vec2,
    pub content_width: u32,
    pub content_height: u32,
}
