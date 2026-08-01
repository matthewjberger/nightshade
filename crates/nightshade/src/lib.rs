pub mod app;
pub mod plugin;

pub use app::{App, Stage};
pub use nightshade_ecs as ecs;
pub use nightshade_ecs::World;
pub use plugin::Plugin;

pub mod prelude {
    pub use crate::app::{App, ParamSystemMarker, Stage, SystemCollection, WorldSystemMarker};
    pub use crate::plugin::{Plugin, PluginGroup, PluginGroupBuilder};
    pub use nalgebra_glm;
    pub use nalgebra_glm::{Mat4, Quat, Vec2, Vec3, Vec4, vec2, vec3, vec4};
    pub use nightshade_ecs as ecs;
    pub use nightshade_ecs::gate::run_if;
    pub use nightshade_ecs::system_param::{
        Added, Changed, Commands, EventReader, EventWriter, MainThread, Query, Res, ResMut, With,
        WithTag, Without, WithoutTag, run_system,
    };
    pub use nightshade_ecs::{Entity, EventBus, HierarchyIndex, World};
    pub use web_time::{Duration, Instant, SystemTime, UNIX_EPOCH};
}
