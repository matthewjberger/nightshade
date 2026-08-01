mod plugin;
pub mod systems;

pub use plugin::ShadowRenderPlugin;

pub mod backend {
    pub use crate::systems::access::shadow_depth_pass_mut;
    pub use crate::systems::shadow_depth::ShadowDepthPass;
}
