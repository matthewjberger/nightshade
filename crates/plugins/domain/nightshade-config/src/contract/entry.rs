#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigKind {
    Toggle {
        default: bool,
    },
    Number {
        minimum: f32,
        maximum: f32,
        default: f32,
    },
    Whole {
        minimum: i64,
        maximum: i64,
        default: i64,
    },
    Choice {
        default: usize,
        options: &'static [&'static str],
    },
    Text {
        default: &'static str,
    },
    Color {
        default: [f32; 4],
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfigEntry {
    pub key: &'static str,
    pub section: &'static str,
    pub kind: ConfigKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigValue {
    Toggle(bool),
    Number(f32),
    Whole(i64),
    Choice(usize),
    Text(String),
    Color([f32; 4]),
}

pub trait ConfigSchema: Copy + Send + Sync + 'static {
    fn entries() -> &'static [ConfigEntry];
    fn index(self) -> usize;
}

impl ConfigKind {
    pub fn default_value(&self) -> ConfigValue {
        match self {
            ConfigKind::Toggle { default } => ConfigValue::Toggle(*default),
            ConfigKind::Number { default, .. } => ConfigValue::Number(*default),
            ConfigKind::Whole { default, .. } => ConfigValue::Whole(*default),
            ConfigKind::Choice { default, .. } => ConfigValue::Choice(*default),
            ConfigKind::Text { default } => ConfigValue::Text((*default).to_string()),
            ConfigKind::Color { default } => ConfigValue::Color(*default),
        }
    }

    pub fn hold(&self, value: ConfigValue) -> ConfigValue {
        match (self, value) {
            (
                ConfigKind::Number {
                    minimum, maximum, ..
                },
                ConfigValue::Number(number),
            ) => ConfigValue::Number(number.clamp(*minimum, *maximum)),
            (
                ConfigKind::Whole {
                    minimum, maximum, ..
                },
                ConfigValue::Whole(number),
            ) => ConfigValue::Whole(number.clamp(*minimum, *maximum)),
            (ConfigKind::Choice { options, .. }, ConfigValue::Choice(index)) => {
                ConfigValue::Choice(index.min(options.len().saturating_sub(1)))
            }
            (ConfigKind::Color { .. }, ConfigValue::Color(channels)) => ConfigValue::Color([
                channels[0].clamp(0.0, 1.0),
                channels[1].clamp(0.0, 1.0),
                channels[2].clamp(0.0, 1.0),
                channels[3].clamp(0.0, 1.0),
            ]),
            (_, other) => other,
        }
    }
}
