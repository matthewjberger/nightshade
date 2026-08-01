use crate::contract::window::Window;

pub fn window_viewport_size(resources: &nightshade_ecs::ResourceMap) -> Option<(u32, u32)> {
    resources.res::<Window>().cached_viewport_size
}

pub fn window_scale_factor(resources: &nightshade_ecs::ResourceMap) -> f32 {
    resources.res::<Window>().cached_scale_factor
}
