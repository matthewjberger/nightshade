pub mod resources;

pub use resources::*;

#[derive(Default)]
pub struct GalleryResources {
    pub state: GalleryState,
}
