use crate::contract::{Config, ConfigEntry, ConfigKind, ConfigSchema, ConfigValue};
use crate::data::CHANNELS;

pub fn config_apply_text<S: ConfigSchema>(config: &mut Config<S>, text: &str) {
    let Ok(table) = text.parse::<toml::Table>() else {
        return;
    };
    for (index, entry) in S::entries().iter().enumerate() {
        let Some(written) = table.get(entry.key) else {
            continue;
        };
        let Some(value) = value_of(entry, written) else {
            continue;
        };
        config.set_at(index, value);
    }
}

fn value_of(entry: &ConfigEntry, written: &toml::Value) -> Option<ConfigValue> {
    match entry.kind {
        ConfigKind::Toggle { .. } => written.as_bool().map(ConfigValue::Toggle),
        ConfigKind::Number { .. } => written
            .as_float()
            .map(|number| number as f32)
            .or_else(|| written.as_integer().map(|number| number as f32))
            .map(ConfigValue::Number),
        ConfigKind::Whole { .. } => written.as_integer().map(ConfigValue::Whole),
        ConfigKind::Choice { options, .. } => written.as_str().and_then(|name| {
            options
                .iter()
                .position(|option| *option == name)
                .map(ConfigValue::Choice)
        }),
        ConfigKind::Text { .. } => written
            .as_str()
            .map(|text| ConfigValue::Text(text.to_string())),
        ConfigKind::Color { .. } => {
            let written = written.as_array()?;
            if written.len() != CHANNELS {
                return None;
            }
            let mut channels = [0.0; CHANNELS];
            for (channel, value) in channels.iter_mut().zip(written) {
                *channel = value
                    .as_float()
                    .map(|number| number as f32)
                    .or_else(|| value.as_integer().map(|number| number as f32))?;
            }
            Some(ConfigValue::Color(channels))
        }
    }
}
