pub mod components;
pub mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::ecs! {
    GalleryWorld {
        screen_marker: ScreenMarker => SCREEN_MARKER,
    }
    Tags {
    }
    Events {
    }
    Resources {
        handles: GalleryHandles,
    }
}
