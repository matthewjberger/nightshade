use crate::contract::input::{Input, Keyboard, Mouse};

pub fn mouse_for_active(resources: &nightshade_ecs::ResourceMap) -> &Mouse {
    &resources.res::<Input>().mouse
}

pub fn keyboard_for_active(resources: &nightshade_ecs::ResourceMap) -> &Keyboard {
    &resources.res::<Input>().keyboard
}
