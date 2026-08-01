use crate::data::WindowArguments;
use nightshade_platform::contract::window::Window;

pub fn size_window(window: &mut Window, arguments: &WindowArguments) {
    if let Some(size) = arguments.size {
        window.initial_size = Some((size.width, size.height));
    }
}
