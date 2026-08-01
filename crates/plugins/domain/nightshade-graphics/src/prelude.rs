pub use nightshade_render::prelude::*;

pub use crate::commands::builtins::{
    despawn_entities_with_cache_cleanup, despawn_recursive_immediate, register_material,
    set_material_with_textures, spawn_cone_at, spawn_cube_at, spawn_cylinder_at,
    spawn_instanced_mesh, spawn_instanced_mesh_with_material, spawn_material, spawn_mesh_at,
    spawn_plane_at, spawn_sphere_at, spawn_sun, spawn_sun_without_shadows, spawn_torus_at,
};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::commands::capture_screenshot;
pub use crate::commands::gpu_picking::request_pick_at;
pub use crate::commands::light::spawn_light_entity;
pub use crate::commands::mesh::spawn_mesh;
pub use crate::commands::{
    load_hdr_skybox, load_hdr_skybox_from_path, queue_ecs_command, queue_render_command,
};
pub use crate::data::queues::{EcsCommand, RenderCommand};
pub use crate::queries::gpu_picking::surface_pick_coords;
pub use crate::resources::queues::CommandQueues;
#[cfg(feature = "plugin")]
pub use crate::systems::bounding_volume::ensure_bounding_volumes;
#[cfg(feature = "plugin")]
pub use crate::systems::mesh::update_instanced_mesh_caches_system;
pub use nightshade_platform::commands::time::{pause, set_paused, set_time_scale, unpause};
pub use nightshade_platform::commands::window::{set_cursor_locked, set_cursor_visible};
pub use nightshade_platform::queries::time::{is_paused, time_scale};
pub use nightshade_render::commands::material_registry::{
    insert_material, material_registry_insert, material_registry_mutate, queue_set_material,
    set_material,
};
pub use nightshade_render::commands::mesh_cache::mesh_cache_insert;
pub use nightshade_render::commands::texture_cache::{
    texture_cache_acquire, texture_cache_protect, texture_cache_release,
    texture_cache_release_entity,
};
pub use nightshade_scene::queries::find_entity_by_name;
