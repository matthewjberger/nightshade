mod core;
mod fly_camera;

pub use core::*;
pub use fly_camera::FlyCameraPlugin;

use crate::ecs::event_bus::EngineEvent;
use crate::ecs::world::World;

pub trait Plugin {
    fn step(&mut self, world: &mut World);

    fn on_event(&mut self, _world: &mut World, _event: &EngineEvent) {}
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

    pub fn push(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn extend(&mut self, other: PluginGroup) {
        self.plugins.extend(other.plugins);
    }

    pub fn on_event(&mut self, world: &mut World, event: &EngineEvent) {
        for plugin in &mut self.plugins {
            plugin.on_event(world, event);
        }
    }

    pub fn step(&mut self, world: &mut World) {
        for plugin in &mut self.plugins {
            plugin.step(world);
        }
    }
}

impl Default for PluginGroup {
    fn default() -> Self {
        Self::new()
    }
}

pub trait PluginGroupExt {
    fn add<P: Plugin + 'static>(self, plugin: P) -> Self;
}

impl PluginGroupExt for PluginGroup {
    fn add<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.push(Box::new(plugin));
        self
    }
}

impl FromIterator<Box<dyn Plugin>> for PluginGroup {
    fn from_iter<T: IntoIterator<Item = Box<dyn Plugin>>>(iter: T) -> Self {
        let mut group = PluginGroup::new();
        for plugin in iter {
            group.push(plugin);
        }
        group
    }
}

pub fn default_plugins() -> PluginGroup {
    PluginGroup::new()
        .add(UIEventPlugin)
        .add(InputEventPlugin)
        .add(TimingPlugin)
        .add(GamepadInputPlugin)
        .add(TransformPlugin)
        .add(CommandProcessingPlugin)
        .add(HudTextSyncPlugin)
        .add(ResetMousePlugin)
}
