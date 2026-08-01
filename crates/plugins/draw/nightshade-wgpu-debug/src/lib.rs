mod plugin;
pub mod systems;

pub use plugin::DebugRenderPlugin;

pub mod backend {
    pub use crate::systems::lines::LinesPass;
}
