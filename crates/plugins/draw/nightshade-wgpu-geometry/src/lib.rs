mod plugin;
pub mod systems;

pub use plugin::GeometryRenderPlugin;

pub mod backend {
    pub use crate::systems::access::{
        MESH_PASS, SCENE_OVERLAY_PASS, SKINNED_MESH_PASS, add_overlay_sub_pass, mesh_pass_mut,
        overlay_sub_pass_mut, scene_overlay_pass_mut, skinned_mesh_pass_mut,
    };
    pub use crate::systems::geometry::OverlaySubPass;
}
