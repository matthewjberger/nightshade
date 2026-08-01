use nightshade_config::contract::{ConfigEntry, ConfigKind, ConfigValue};
use nightshade_ui::prelude::{
    UiCheckboxData, UiColorPickerData, UiDragValueData, UiDropdownData, UiSliderData,
    UiTextInputData,
};

pub type WidgetRow<'a> = (
    Option<&'a UiSliderData>,
    Option<&'a UiCheckboxData>,
    Option<&'a UiDragValueData>,
    Option<&'a UiDropdownData>,
    Option<&'a UiTextInputData>,
    Option<&'a UiColorPickerData>,
);

pub fn row_value(entry: &ConfigEntry, row: WidgetRow<'_>) -> Option<ConfigValue> {
    let (slider, checkbox, drag, dropdown, input, picker) = row;
    match entry.kind {
        ConfigKind::Toggle { .. } => checkbox.map(|checkbox| ConfigValue::Toggle(checkbox.value)),
        ConfigKind::Number { .. } => slider.map(|slider| ConfigValue::Number(slider.value)),
        ConfigKind::Whole { .. } => drag.map(|drag| ConfigValue::Whole(drag.value.round() as i64)),
        ConfigKind::Choice { .. } => {
            dropdown.map(|dropdown| ConfigValue::Choice(dropdown.selected_index))
        }
        ConfigKind::Text { .. } => input.map(|input| ConfigValue::Text(input.text.clone())),
        ConfigKind::Color { .. } => picker.map(|picker| {
            ConfigValue::Color([
                picker.color.x,
                picker.color.y,
                picker.color.z,
                picker.color.w,
            ])
        }),
    }
}
