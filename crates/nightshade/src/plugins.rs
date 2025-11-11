mod fly_camera;

pub use fly_camera::FlyCameraPlugin;

use crate::ecs::world::World;

pub trait Plugin {
    fn update(&mut self, world: &mut World);
}

pub fn run_plugins(plugins: &mut [Box<dyn Plugin>], world: &mut World) {
    for plugin in plugins {
        plugin.update(world);
    }
}

pub struct PluginGroup {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginGroup {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }
}

impl Default for PluginGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PluginGroup {
    fn update(&mut self, world: &mut World) {
        for plugin in &mut self.plugins {
            plugin.update(world);
        }
    }
}
