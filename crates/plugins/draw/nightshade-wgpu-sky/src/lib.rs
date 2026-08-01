mod plugin;
pub mod systems;

pub use plugin::SkyRenderPlugin;

pub mod backend {
    pub use crate::systems::access::sky_pass_mut;
    pub use crate::systems::sky::SkyPass;
}
