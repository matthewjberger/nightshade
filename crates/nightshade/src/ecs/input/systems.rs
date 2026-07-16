use crate::ecs::world::{Vec2, World, resources::MouseState};

pub fn apply_ime_allowed_system(world: &mut World) {
    let _span = tracing::info_span!("apply_ime_allowed").entered();
    let (allowed, applied) = {
        let ime = &world
            .res::<crate::ecs::input::resources::Input>()
            .keyboard
            .ime;
        (ime.allowed, ime.applied)
    };
    if allowed == applied {
        return;
    }
    let Some(window) = world
        .res::<crate::ecs::window::resources::Window>()
        .handle
        .as_ref()
    else {
        return;
    };
    window.set_ime_allowed(allowed);
    world
        .res_mut::<crate::ecs::input::resources::Input>()
        .keyboard
        .ime
        .applied = allowed;
}

pub fn set_ime_allowed(world: &mut World, allowed: bool) {
    world
        .res_mut::<crate::ecs::input::resources::Input>()
        .keyboard
        .ime
        .allowed = allowed;
}

pub fn reset_mouse_system(world: &mut World) {
    let _span = tracing::info_span!("reset_mouse").entered();
    let mouse = &mut world.res_mut::<crate::ecs::input::resources::Input>().mouse;
    if mouse.state.contains(MouseState::SCROLLED) {
        mouse.wheel_delta = Vec2::new(0.0, 0.0);
    }
    mouse.state.set(MouseState::SCROLLED, false);
    if mouse.state.contains(MouseState::MOVED) {
        mouse.position_delta = Vec2::new(0.0, 0.0);
    }
    mouse.state.set(MouseState::MOVED, false);
    mouse.raw_mouse_delta = Vec2::new(0.0, 0.0);
    mouse.state.set(MouseState::LEFT_JUST_PRESSED, false);
    mouse.state.set(MouseState::MIDDLE_JUST_PRESSED, false);
    mouse.state.set(MouseState::RIGHT_JUST_PRESSED, false);
    mouse.state.set(MouseState::LEFT_JUST_RELEASED, false);
    mouse.state.set(MouseState::MIDDLE_JUST_RELEASED, false);
    mouse.state.set(MouseState::RIGHT_JUST_RELEASED, false);
}

pub fn reset_touch_system(world: &mut World) {
    let _span = tracing::info_span!("reset_touch").entered();
    use crate::ecs::input::resources::TouchGesture;

    let touch = &mut world.res_mut::<crate::ecs::input::resources::Input>().touch;
    touch.clear_ended_touches();
    touch.gesture = TouchGesture::None;
}

pub fn reset_keyboard_system(world: &mut World) {
    let keyboard = &mut world
        .res_mut::<crate::ecs::input::resources::Input>()
        .keyboard;
    keyboard.just_pressed_keys.clear();
    keyboard.just_released_keys.clear();
    keyboard.frame_chars.clear();
    keyboard.frame_keys.clear();
}

pub fn escape_key_exit_system(world: &mut World) {
    if crate::ecs::input::access::keyboard_for_active(world)
        .is_key_pressed(winit::keyboard::KeyCode::Escape)
    {
        world
            .res_mut::<crate::ecs::window::resources::Window>()
            .should_exit = true;
    }
}
