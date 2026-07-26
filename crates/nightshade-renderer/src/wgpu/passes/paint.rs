//! The passes that draw a `nightshade-paint` draw list over the scene: the
//! batched rectangle and text pass, and the textured quad pass.

mod image;
pub use image::PaintImagePass;

mod draw_list;
pub use draw_list::PaintPass;
