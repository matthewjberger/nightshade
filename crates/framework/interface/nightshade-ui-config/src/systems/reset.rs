use crate::resources::ConfigPanel;
use nightshade::prelude::{Commands, Res, ResMut};
use nightshade_config::contract::{Config, ConfigSchema, ConfigValue};
use nightshade_ui::prelude::{
    RetainedUiFrame, UiEvent, Vec4, ui_checkbox_set_value, ui_color_picker_set_value,
    ui_drag_value_set_value, ui_dropdown_set_value, ui_events, ui_slider_set_value,
    ui_text_input_set_value,
};

pub fn follow_the_reset<S: ConfigSchema>(
    mut config: ResMut<Config<S>>,
    panel: Res<ConfigPanel<S>>,
    frame: Res<RetainedUiFrame>,
    mut commands: Commands,
) {
    let Some(reset) = panel.reset else {
        return;
    };
    let clicked = ui_events(&frame)
        .iter()
        .any(|event| matches!(event, UiEvent::ButtonClicked(entity) if *entity == reset));
    if !clicked {
        return;
    }

    config.reset_all();
    for (index, row) in panel.rows.iter().copied().enumerate() {
        let Some(value) = config.values().get(index).cloned() else {
            continue;
        };
        match &value {
            ConfigValue::Toggle(state) => ui_checkbox_set_value(&mut commands, row, *state),
            ConfigValue::Number(number) => ui_slider_set_value(&mut commands, row, *number),
            ConfigValue::Whole(number) => {
                ui_drag_value_set_value(&mut commands, row, *number as f32)
            }
            ConfigValue::Choice(choice) => ui_dropdown_set_value(&mut commands, row, *choice),
            ConfigValue::Text(text) => ui_text_input_set_value(&mut commands, row, text),
            ConfigValue::Color(channels) => ui_color_picker_set_value(
                &mut commands,
                row,
                Vec4::new(channels[0], channels[1], channels[2], channels[3]),
            ),
        }
    }
}
