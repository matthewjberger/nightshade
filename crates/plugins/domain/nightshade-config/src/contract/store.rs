use crate::contract::entry::{ConfigEntry, ConfigSchema, ConfigValue};
use std::marker::PhantomData;

pub struct Config<S> {
    values: Vec<ConfigValue>,
    revision: u64,
    schema: PhantomData<fn() -> S>,
}

impl<S: ConfigSchema> Default for Config<S> {
    fn default() -> Self {
        Self {
            values: S::entries()
                .iter()
                .map(|entry| entry.kind.default_value())
                .collect(),
            revision: 0,
            schema: PhantomData,
        }
    }
}

impl<S: ConfigSchema> Config<S> {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn values(&self) -> &[ConfigValue] {
        &self.values
    }

    pub fn entry(&self, key: S) -> &'static ConfigEntry {
        &S::entries()[key.index()]
    }

    pub fn value(&self, key: S) -> &ConfigValue {
        &self.values[key.index()]
    }

    pub fn toggle(&self, key: S) -> bool {
        match self.value(key) {
            ConfigValue::Toggle(state) => *state,
            _ => wrong_kind(self.entry(key).key, "toggle"),
        }
    }

    pub fn number(&self, key: S) -> f32 {
        match self.value(key) {
            ConfigValue::Number(number) => *number,
            _ => wrong_kind(self.entry(key).key, "number"),
        }
    }

    pub fn whole(&self, key: S) -> i64 {
        match self.value(key) {
            ConfigValue::Whole(number) => *number,
            _ => wrong_kind(self.entry(key).key, "whole"),
        }
    }

    pub fn choice(&self, key: S) -> usize {
        match self.value(key) {
            ConfigValue::Choice(index) => *index,
            _ => wrong_kind(self.entry(key).key, "choice"),
        }
    }

    pub fn text(&self, key: S) -> &str {
        match self.value(key) {
            ConfigValue::Text(text) => text,
            _ => wrong_kind(self.entry(key).key, "text"),
        }
    }

    pub fn color(&self, key: S) -> [f32; 4] {
        match self.value(key) {
            ConfigValue::Color(channels) => *channels,
            _ => wrong_kind(self.entry(key).key, "color"),
        }
    }

    pub fn option(&self, key: S) -> &'static str {
        let index = self.choice(key);
        options_of(self.entry(key))
            .get(index)
            .copied()
            .unwrap_or("")
    }

    pub fn set_toggle(&mut self, key: S, state: bool) {
        self.set(key, ConfigValue::Toggle(state));
    }

    pub fn set_number(&mut self, key: S, number: f32) {
        self.set(key, ConfigValue::Number(number));
    }

    pub fn set_whole(&mut self, key: S, number: i64) {
        self.set(key, ConfigValue::Whole(number));
    }

    pub fn set_choice(&mut self, key: S, index: usize) {
        self.set(key, ConfigValue::Choice(index));
    }

    pub fn set_text(&mut self, key: S, text: impl Into<String>) {
        self.set(key, ConfigValue::Text(text.into()));
    }

    pub fn set_color(&mut self, key: S, channels: [f32; 4]) {
        self.set(key, ConfigValue::Color(channels));
    }

    pub fn set(&mut self, key: S, value: ConfigValue) {
        self.set_at(key.index(), value);
    }

    pub fn set_at(&mut self, index: usize, value: ConfigValue) {
        let Some(entry) = S::entries().get(index) else {
            return;
        };
        let value = entry.kind.hold(value);
        if self.values[index] == value {
            return;
        }
        self.values[index] = value;
        self.revision += 1;
    }

    pub fn reset(&mut self, key: S) {
        let index = key.index();
        self.set_at(index, S::entries()[index].kind.default_value());
    }

    pub fn reset_all(&mut self) {
        for (index, entry) in S::entries().iter().enumerate() {
            self.set_at(index, entry.kind.default_value());
        }
    }
}

pub fn options_of(entry: &ConfigEntry) -> &'static [&'static str] {
    match entry.kind {
        crate::contract::entry::ConfigKind::Choice { options, .. } => options,
        _ => &[],
    }
}

fn wrong_kind(key: &str, wanted: &str) -> ! {
    panic!("the setting {key} is not a {wanted}")
}
