use crate::commands::config_apply_text;
use crate::contract::{Config, ConfigSchema};
use crate::queries::config_text;
use crate::resources::ConfigFile;
use crate::systems::persist::keep_the_settings;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_store::prelude::{store_read_text, store_write_text};
use std::marker::PhantomData;

pub struct ConfigPlugin<S> {
    file: Option<(&'static str, &'static str)>,
    schema: PhantomData<fn() -> S>,
}

impl<S> ConfigPlugin<S> {
    pub fn new() -> Self {
        Self {
            file: None,
            schema: PhantomData,
        }
    }

    pub fn persist(mut self, folder: &'static str, name: &'static str) -> Self {
        self.file = Some((folder, name));
        self
    }
}

impl<S> Default for ConfigPlugin<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: ConfigSchema> Plugin for ConfigPlugin<S> {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        let mut config = Config::<S>::default();
        if let Some((folder, name)) = self.file {
            match store_read_text(folder, name) {
                Some(text) => config_apply_text(&mut config, &text),
                None => {
                    store_write_text(folder, name, &config_text(&config));
                }
            }
            let mut file = ConfigFile::<S>::new(folder, name);
            file.saved_revision = config.revision();
            app.insert_resource(file);
            app.add_system(Stage::Last, keep_the_settings::<S>);
        }
        app.insert_resource(config);
    }
}
