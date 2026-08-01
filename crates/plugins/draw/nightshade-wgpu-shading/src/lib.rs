mod plugin;
mod systems;

pub use plugin::ShadingPlugin;

pub mod backend {
    pub use crate::systems::cluster;
    pub use crate::systems::fog;
    pub use crate::systems::hiz;
    pub use crate::systems::light;
    pub use crate::systems::ltc;
    pub use crate::systems::material_gpu;
    pub use crate::systems::modules::{DEPTH_CLEAR, SKINNING_COMPUTE};
    pub use crate::systems::projection;
    pub use crate::systems::scene_uniforms;
    pub use crate::systems::shadow_inputs;
}
