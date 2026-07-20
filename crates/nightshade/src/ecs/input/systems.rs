use crate::ecs::world::World;

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
    world
        .res_mut::<crate::ecs::input::resources::Input>()
        .mouse
        .end_frame();
}

pub fn reset_touch_system(world: &mut World) {
    let _span = tracing::info_span!("reset_touch").entered();
    world
        .res_mut::<crate::ecs::input::resources::Input>()
        .touch
        .end_frame();
}

pub fn reset_keyboard_system(world: &mut World) {
    world
        .res_mut::<crate::ecs::input::resources::Input>()
        .keyboard
        .end_frame();
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
