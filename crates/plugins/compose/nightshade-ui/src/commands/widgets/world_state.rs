use nalgebra_glm;
use nalgebra_glm::Vec2;

use crate::data::units::Rl;

use crate::prelude::*;
pub fn ui_clicked(
    buttons: &mut nightshade::ecs::system_param::Query<'_, (&crate::components::UiButtonData,)>,
    entity: impl Into<Option<nightshade::ecs::Entity>>,
) -> bool {
    let Some(entity) = entity.into() else {
        return false;
    };
    let mut clicked = false;
    buttons.for_entity(entity, |(data,)| clicked = data.clicked);
    clicked
}

pub fn measure_ui_text_width(
    text_state: &mut nightshade_text::contract::resources::TextState,
    text: &str,
    font_size: f32,
) -> f32 {
    crate::data::units::measure_text_width(&mut text_state.font_engine, text, font_size)
}

pub fn widget<'a, T: crate::components::WidgetData + Send + Sync + 'static>(
    commands: &'a nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
) -> Option<&'a T> {
    commands.staged::<T>(entity)
}

pub fn widget_in_world<T: crate::components::WidgetData>(
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<&T> {
    T::get(storage, entity)
}

pub fn ui_slider_value(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<f32> {
    world
        .get::<crate::components::UiSliderData>(entity)
        .map(|data| data.value)
}

pub fn ui_slider_value_changed(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<f32> {
    if let Some(data) = world.get::<crate::components::UiSliderData>(entity)
        && data.changed
    {
        Some(data.value)
    } else {
        None
    }
}

pub fn ui_checkbox_value(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<bool> {
    world
        .get::<crate::components::UiCheckboxData>(entity)
        .map(|data| data.value)
}

pub fn ui_checkbox_changed(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<bool> {
    if let Some(data) = world.get::<crate::components::UiCheckboxData>(entity)
        && data.changed
    {
        Some(data.value)
    } else {
        None
    }
}

pub fn ui_text_input_text(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<&str> {
    if let Some(data) = world.get::<crate::components::UiTextInputData>(entity) {
        Some(data.text.as_str())
    } else {
        None
    }
}

pub fn ui_dropdown_selected(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> Option<usize> {
    world
        .get::<crate::components::UiDropdownData>(entity)
        .map(|data| data.selected_index)
}

pub fn ui_button_set_text(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    text: &str,
) {
    let text = text.to_string();
    commands.queue(move |storage, resources| {
        let text = text.as_str();

        if let Some(data) = storage.get::<crate::components::UiButtonData>(entity)
            && let Some(slot) = data.text_slot
        {
            resources
                .res_mut::<nightshade_text::contract::resources::TextState>()
                .cache
                .set_text(slot, text);
        }
    });
}

pub fn ui_slider_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    value: f32,
) {
    commands.queue(move |storage, resources| {
        write_slider(storage, resources, entity, value);
    });
}

fn write_slider(
    storage: &mut nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
    entity: nightshade::ecs::Entity,
    value: f32,
) {
    let update = if let Some(data) = storage.get::<crate::components::UiSliderData>(entity) {
        let normalized = if data.logarithmic && data.min > 0.0 && data.max > data.min {
            ((value / data.min).ln() / (data.max / data.min).ln()).clamp(0.0, 1.0)
        } else {
            ((value - data.min) / (data.max - data.min)).clamp(0.0, 1.0)
        };
        Some((
            data.fill_entity,
            data.text_slot,
            data.min,
            data.max,
            normalized,
            data.logarithmic,
            data.prefix.clone(),
            data.suffix.clone(),
            data.precision,
        ))
    } else {
        None
    };

    let Some((
        fill_entity,
        text_slot,
        min,
        max,
        normalized,
        logarithmic,
        prefix,
        suffix,
        precision,
    )) = update
    else {
        return;
    };
    let clamped = if logarithmic && min > 0.0 && max > min {
        min * (max / min).powf(normalized)
    } else {
        min + normalized * (max - min)
    };
    if let Some(fill_node) = storage.get_mut::<crate::components::UiLayoutNode>(fill_entity)
        && let Some(crate::components::layout_types::UiLayoutType::Boundary(boundary)) =
            fill_node.base_layout.as_mut()
    {
        boundary.position_2 = Rl(Vec2::new(normalized * 100.0, 100.0)).into();
    }
    resources
        .res_mut::<nightshade_text::contract::resources::TextState>()
        .cache
        .set_text(text_slot, format!("{prefix}{clamped:.precision$}{suffix}"));
    if let Some(data) = storage.get_mut::<crate::components::UiSliderData>(entity) {
        data.value = clamped;
    }
}

pub fn ui_checkbox_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    value: bool,
) {
    commands.queue(move |storage, _| {
        let inner = storage
            .get_mut::<crate::components::UiCheckboxData>(entity)
            .map(|data| {
                data.value = value;
                data.inner_entity
            });
        if let Some(inner) = inner
            && let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(inner)
        {
            node.visible = value;
        }
    });
}

pub fn ui_color_picker_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    color: nalgebra_glm::Vec4,
) {
    commands.queue(move |storage, resources| {
        let picker = storage
            .get_mut::<crate::components::UiColorPickerData>(entity)
            .map(|data| {
                data.color = color;
                (data.swatch_entity, data.slider_entities, data.mode)
            });
        let Some((swatch_entity, slider_entities, mode)) = picker else {
            return;
        };
        if let Some(swatch) = storage.get_mut::<crate::components::UiNodeColor>(swatch_entity) {
            swatch.colors[crate::data::state::UiBase::INDEX] = Some(color);
        }
        let values = match mode {
            crate::components::ColorPickerMode::Rgb => [color.x, color.y, color.z, color.w],
            crate::components::ColorPickerMode::Hsv => {
                let hsva = crate::data::color::Hsva::from_rgba(color);
                [hsva.hue, hsva.saturation, hsva.value, color.w]
            }
        };
        for (slider_entity, value) in slider_entities.iter().zip(values) {
            write_slider(storage, resources, *slider_entity, value);
        }
    });
}

pub fn ui_drag_value_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    value: f32,
) {
    commands.queue(move |storage, resources| {
        let update = storage
            .get_mut::<crate::components::UiDragValueData>(entity)
            .and_then(|data| {
                if (data.value - value).abs() < 1.0e-5 {
                    return None;
                }
                data.value = value.clamp(data.min, data.max);
                Some((
                    data.text_slot,
                    format!(
                        "{}{:.prec$}{}",
                        data.prefix,
                        data.value,
                        data.suffix,
                        prec = data.precision
                    ),
                ))
            });
        if let Some((slot, text)) = update {
            resources
                .res_mut::<nightshade_text::contract::resources::TextState>()
                .cache
                .set_text(slot, &text);
        }
    });
}

pub fn ui_toggle_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    value: bool,
) {
    commands.queue(move |storage, _| {
        if let Some(data) = storage.get_mut::<crate::components::UiToggleData>(entity) {
            data.value = value;
        }
    });
}

pub fn ui_progress_bar_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    value: f32,
) {
    commands.queue(move |storage, resources| {
        let update = storage
            .get::<crate::components::UiProgressBarData>(entity)
            .map(|data| data.fill_entity);

        if let Some(fill_entity) = update {
            let normalized = value.clamp(0.0, 1.0);
            if let Some(fill_node) = storage.get_mut::<crate::components::UiLayoutNode>(fill_entity)
                && let Some(crate::components::layout_types::UiLayoutType::Boundary(boundary)) =
                    fill_node.base_layout.as_mut()
            {
                boundary.position_2 = Rl(Vec2::new(normalized * 100.0, 100.0)).into();
            }
            if let Some(data) = storage.get_mut::<crate::components::UiProgressBarData>(entity) {
                data.value = normalized;
            }
            ui_mark_layout_dirty(resources.res_mut::<crate::resources::RetainedUiDirty>());
        }
    });
}

pub fn ui_scroll_area_set_offset(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    offset: f32,
) {
    commands.queue(move |storage, resources| {
        if let Some(data) = storage.get_mut::<crate::components::UiScrollAreaData>(entity) {
            data.scroll_offset = offset;
            ui_mark_layout_dirty(resources.res_mut::<crate::resources::RetainedUiDirty>());
        }
    });
}

pub fn ui_virtual_list_set_count(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    count: usize,
) {
    commands.queue(move |storage, resources| {
        if let Some(data) = storage.get_mut::<crate::components::UiVirtualListData>(entity) {
            data.total_items = count;
            ui_mark_layout_dirty(resources.res_mut::<crate::resources::RetainedUiDirty>());
        }
    });
}

pub fn ui_text_input_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    text: &str,
) {
    let text = text.to_string();
    commands.queue(move |storage, resources| {
        let text = text.as_str();

        let (text_slot, placeholder_entity, password) =
            if let Some(data) = storage.get::<crate::components::UiTextInputData>(entity) {
                (Some(data.text_slot), data.placeholder_entity, data.password)
            } else {
                (None, None, false)
            };

        if let Some(slot) = text_slot {
            resources
                .res_mut::<nightshade_text::contract::resources::TextState>()
                .cache
                .set_text(slot, masked_input_display(text, password));
            if let Some(data) = storage.get_mut::<crate::components::UiTextInputData>(entity) {
                data.text = text.to_string();
                data.cursor_position = text.chars().count();
                data.selection_start = None;
            }
            if let Some(ph_entity) = placeholder_entity
                && let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(ph_entity)
            {
                node.visible = text.is_empty();
            }
        }
    });
}

pub fn ui_dropdown_set_value(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    index: usize,
) {
    commands.queue(move |storage, resources| {
        let update = if let Some(data) = storage.get::<crate::components::UiDropdownData>(entity) {
            if index < data.options.len() && index != data.selected_index {
                Some((data.header_text_slot, data.options[index].clone()))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((text_slot, selected_text)) = update {
            resources
                .res_mut::<nightshade_text::contract::resources::TextState>()
                .cache
                .set_text(text_slot, selected_text);
            if let Some(data) = storage.get_mut::<crate::components::UiDropdownData>(entity) {
                data.selected_index = index;
            }
        }
    });
}

pub fn ui_is_text_editor(
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> bool {
    storage
        .get::<crate::components::UiTextInputData>(entity)
        .is_some()
        || storage
            .get::<crate::components::UiTextAreaData>(entity)
            .is_some()
        || storage
            .get::<crate::components::UiDragValueData>(entity)
            .is_some()
}

pub fn ui_pointer_target(
    resources: &nightshade::ecs::ResourceMap,
    storage: &nightshade::ecs::Storage,
) -> Option<nightshade::ecs::Entity> {
    ui_pointer_target_of(
        resources.res::<crate::resources::RetainedUiRuntime>(),
        resources.res::<crate::resources::RetainedUiInteraction>(),
        storage,
    )
}

pub fn ui_pointer_target_of(
    runtime: &crate::resources::RetainedUiRuntime,
    interaction: &crate::resources::RetainedUiInteraction,
    storage: &nightshade::ecs::Storage,
) -> Option<nightshade::ecs::Entity> {
    if !runtime.visible {
        return None;
    }
    interaction
        .hovered_entity
        .or(interaction.active_entity)
        .or_else(|| ui_dragging_tile_container(storage))
}

fn ui_dragging_tile_container(world: &nightshade::ecs::Storage) -> Option<nightshade::ecs::Entity> {
    world
        .query_entities::<(crate::components::UiTileContainerData,)>()
        .find(|entity| {
            world
                .get::<crate::components::UiTileContainerData>(*entity)
                .is_some_and(|data| data.dragging_splitter.is_some() || data.dragging_tab.is_some())
        })
}

pub fn ui_wants_pointer(
    resources: &nightshade::ecs::ResourceMap,
    storage: &nightshade::ecs::Storage,
) -> bool {
    ui_pointer_target(resources, storage).is_some()
}

pub fn ui_wants_pointer_of(
    runtime: &crate::resources::RetainedUiRuntime,
    interaction: &crate::resources::RetainedUiInteraction,
    storage: &nightshade::ecs::Storage,
) -> bool {
    ui_pointer_target_of(runtime, interaction, storage).is_some()
}

pub fn ui_wants_keyboard(
    resources: &nightshade::ecs::ResourceMap,
    storage: &nightshade::ecs::Storage,
) -> bool {
    if !resources
        .res::<crate::resources::RetainedUiRuntime>()
        .visible
    {
        return false;
    }
    resources
        .res::<crate::resources::RetainedUiInteraction>()
        .focused_entity
        .is_some_and(|focused| ui_is_text_editor(storage, focused))
}

pub(crate) fn masked_input_display(text: &str, password: bool) -> String {
    if password {
        "*".repeat(text.chars().count())
    } else {
        text.to_string()
    }
}
