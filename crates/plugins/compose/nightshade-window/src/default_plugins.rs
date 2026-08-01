use nightshade::plugin::{Plugin, PluginGroup, PluginGroupBuilder};
use std::marker::PhantomData;

pub struct DefaultPlugins<A, R> {
    arguments: PhantomData<fn() -> A>,
    renderer: PhantomData<fn() -> R>,
}

impl<A, R> Default for DefaultPlugins<A, R> {
    fn default() -> Self {
        Self {
            arguments: PhantomData,
            renderer: PhantomData,
        }
    }
}

impl<A, R> DefaultPlugins<A, R> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<A, R> PluginGroup for DefaultPlugins<A, R>
where
    A: nightshade_cli::clap::Args + Default + Send + Sync + 'static,
    R: Plugin + Default + 'static,
{
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::new()
            .add_plugin(crate::plugin::WindowPlugin::default())
            .add_plugin(nightshade_cli::CliPlugin::<A>::default())
            .add_plugin(nightshade_log::LogPlugin::default())
            .add_plugin(R::default())
            .add_plugin(nightshade_camera::CameraPlugin)
    }
}
