//! The passes that draw a `nightshade-paint` draw list over the finished
//! scene: the batched rectangle and text pass, and the image pass that blits
//! textured quads into the same layers.

mod image;
pub use image::PaintImagePass;

mod draw_list;
pub use draw_list::PaintPass;
