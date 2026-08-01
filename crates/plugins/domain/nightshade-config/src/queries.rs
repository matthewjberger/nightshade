use crate::contract::{Config, ConfigSchema, ConfigValue, options_of};

pub fn config_text<S: ConfigSchema>(config: &Config<S>) -> String {
    let mut table = toml::Table::new();
    for (index, entry) in S::entries().iter().enumerate() {
        let written = match &config.values()[index] {
            ConfigValue::Toggle(state) => toml::Value::Boolean(*state),
            ConfigValue::Number(number) => toml::Value::Float(printed_number(*number)),
            ConfigValue::Whole(number) => toml::Value::Integer(*number),
            ConfigValue::Choice(choice) => toml::Value::String(
                options_of(entry)
                    .get(*choice)
                    .copied()
                    .unwrap_or("")
                    .to_string(),
            ),
            ConfigValue::Text(text) => toml::Value::String(text.clone()),
            ConfigValue::Color(channels) => toml::Value::Array(
                channels
                    .iter()
                    .map(|channel| toml::Value::Float(printed_number(*channel)))
                    .collect(),
            ),
        };
        table.insert(entry.key.to_string(), written);
    }
    toml::to_string(&table).unwrap_or_default()
}

fn printed_number(number: f32) -> f64 {
    format!("{number}").parse().unwrap_or(number as f64)
}
