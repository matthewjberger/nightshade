use crate::resources::ConfigPanel;
use crate::systems::reset::follow_the_reset;
use crate::systems::rows::read_the_rows;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_config::contract::ConfigSchema;
use std::marker::PhantomData;

pub struct ConfigUiPlugin<S> {
    schema: PhantomData<fn() -> S>,
}

impl<S> ConfigUiPlugin<S> {
    pub fn new() -> Self {
        Self {
            schema: PhantomData,
        }
    }
}

impl<S> Default for ConfigUiPlugin<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: ConfigSchema> Plugin for ConfigUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConfigPanel::<S>::default());
        app.add_system(Stage::Update, read_the_rows::<S>);
        app.add_system(Stage::Update, follow_the_reset::<S>);
    }
}
