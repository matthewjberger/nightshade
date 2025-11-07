pub mod ecs;
pub mod render;
pub mod run;
mod systems;

pub use systems::*;

pub mod prelude {
    pub use crate::ecs::world::components::*;
    pub use crate::ecs::world::resources::*;
    pub use crate::ecs::world::*;
    pub use crate::run::*;
    pub use crate::systems::*;
    pub use web_time::{Duration, Instant, SystemTime};

    pub use crate::ecs::event_bus::commands::{publish_app_event, publish_event};
    pub use crate::ecs::mesh::commands::spawn_mesh;
    pub use crate::ecs::text::commands::load_font_from_bytes;
    pub use egui;
    pub use freecs;
    pub use log;
    pub use rand::{self, Rng};
    pub use serde;
    pub use winit::{
        event::{ElementState as KeyState, WindowEvent},
        keyboard::KeyCode,
    };

    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::passes;
    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::rendergraph::{
        PassExecutionContext, PassNode, RenderGraph, ResourceId,
    };
    pub use nalgebra_glm;
    pub use wgpu;
}
