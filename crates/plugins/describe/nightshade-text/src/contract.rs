pub mod components;
pub mod data;
pub mod render;
pub mod resources;

pub use components::{Text, TextCharacterBackgroundColors, TextCharacterColors, TextProperties};
pub use data::font_engine::{DEFAULT_FONT_DATA, DEFAULT_MONO_FONT_DATA, FontEngine};
pub use data::text_data::{FontKind, TextAlignment, TextMesh, TextVertex, VerticalAlignment};
pub use render::RenderText;
pub use resources::{TextCache, TextState};
