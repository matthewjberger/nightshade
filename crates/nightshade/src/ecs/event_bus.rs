pub mod commands;
pub mod resources;
pub mod systems;

pub use commands::*;
pub use resources::*;
pub use systems::*;

use winit::event::WindowEvent;

pub enum EngineEvent<'a> {
    Window(&'a WindowEvent),
}
