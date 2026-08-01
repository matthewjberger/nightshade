use crate::resources::BannerHandles;
use nightshade::ecs::Entity;

pub fn built(handles: &BannerHandles) -> bool {
    handles.text != Entity::default()
}
