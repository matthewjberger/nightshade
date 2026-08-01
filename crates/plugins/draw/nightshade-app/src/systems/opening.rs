use crate::resources::Opening;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::window::Window;

pub fn name_the_window(opening: Res<Opening>, mut window: ResMut<Window>) {
    window.title = opening.title.to_string();
    if let Some(size) = opening.size {
        window.initial_size = Some(size);
    }
}
