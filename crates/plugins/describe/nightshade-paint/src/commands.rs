use crate::resources::{PaintDrawList, PaintFrame, PaintTextCache, RenderSlotAllocator};
use nightshade::ecs::ResourceMap;

pub fn install_paint_resources(resources: &mut ResourceMap) {
    hold::<PaintFrame>(resources);
    hold::<PaintDrawList>(resources);
    hold::<RenderSlotAllocator>(resources);
    hold::<PaintTextCache>(resources);
}

fn hold<T: Default + Send + Sync + 'static>(resources: &mut ResourceMap) {
    if resources.get::<T>().is_none() {
        resources.insert(T::default());
    }
}
