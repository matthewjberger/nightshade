use crate::data::{RESET_LABEL, WHOLE_SPEED};
use crate::resources::ConfigPanel;
use nightshade::ecs::Entity;
use nightshade_config::contract::{Config, ConfigKind, ConfigSchema, ConfigValue, config_label};
use nightshade_ui::prelude::{DragValueConfig, UiTreeBuilder, Vec4};

pub fn build_config_rows<S: ConfigSchema>(
    tree: &mut UiTreeBuilder<'_, '_>,
    config: &Config<S>,
) -> ConfigPanel<S> {
    let mut panel = ConfigPanel::<S>::default();
    let mut section = "";
    for (index, entry) in S::entries().iter().enumerate() {
        if entry.section != section {
            section = entry.section;
            tree.add_heading(section);
        }
        let label = config_label(entry.key);
        let row = match (&entry.kind, &config.values()[index]) {
            (ConfigKind::Toggle { .. }, ConfigValue::Toggle(state)) => {
                tree.add_checkbox(&label, *state)
            }
            (
                ConfigKind::Number {
                    minimum, maximum, ..
                },
                ConfigValue::Number(number),
            ) => {
                tree.add_label(&label);
                tree.add_slider(*minimum, *maximum, *number)
            }
            (
                ConfigKind::Whole {
                    minimum, maximum, ..
                },
                ConfigValue::Whole(number),
            ) => {
                tree.add_label(&label);
                tree.add_drag_value_configured(
                    DragValueConfig::new(*minimum as f32, *maximum as f32, *number as f32)
                        .precision(0)
                        .speed(WHOLE_SPEED),
                )
            }
            (ConfigKind::Choice { options, .. }, ConfigValue::Choice(choice)) => {
                tree.add_label(&label);
                tree.add_dropdown(options, *choice)
            }
            (ConfigKind::Text { .. }, ConfigValue::Text(text)) => {
                tree.add_label(&label);
                tree.add_text_input_with_value(&label, text)
            }
            (ConfigKind::Color { .. }, ConfigValue::Color(channels)) => {
                tree.add_label(&label);
                tree.add_color_picker(Vec4::new(
                    channels[0],
                    channels[1],
                    channels[2],
                    channels[3],
                ))
            }
            _ => Entity::default(),
        };
        panel.rows.push(row);
    }
    panel.reset = Some(tree.add_button(RESET_LABEL));
    panel
}
