pub mod ecs;
pub mod plugins;
pub mod render;
pub mod run;
mod systems;

pub use systems::*;

pub mod prelude {
    pub use crate::ecs::world::components::*;
    pub use crate::ecs::world::resources::*;
    pub use crate::ecs::world::*;
    pub use crate::plugins::*;
    pub use crate::run::*;
    pub use crate::systems::*;
    pub use web_time::{Duration, Instant, SystemTime};

    pub const LOGO_BYTES: &[u8] = include_bytes!("../images/logo.png");

    pub use crate::ecs::event_bus::commands::{publish_app_event, publish_event};
    pub use crate::ecs::mesh::commands::spawn_mesh;
    pub use crate::ecs::picking::{
        PickingOptions, PickingRay, PickingResult, get_ground_position_from_screen,
        pick_closest_entity, pick_entities, pick_entities_in_frustum, pick_entities_with_options,
    };
    pub use crate::ecs::splash_screen::commands::{
        is_splash_screen_completed, spawn_splash_screen_quad, spawn_splash_screen_sprite,
        spawn_splash_screen_text,
    };
    pub use crate::ecs::text::commands::{
        load_font_from_bytes, spawn_hud_text, spawn_hud_text_with_properties,
    };
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
