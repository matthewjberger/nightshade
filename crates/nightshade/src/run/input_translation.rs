use crate::ecs::world::World;
use crate::prelude::*;
use winit::event::WindowEvent;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn recenter_locked_cursor(world: &mut World) {
    if !world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .cursor_locked
    {
        return;
    }
    let Some(window_handle) = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .handle
        .as_ref()
    else {
        return;
    };
    let size = window_handle.inner_size();
    if size.width == 0 || size.height == 0 {
        return;
    }
    window_handle.set_cursor_visible(false);
    let center_x = size.width as f32 / 2.0;
    let center_y = size.height as f32 / 2.0;
    if window_handle
        .set_cursor_position(winit::dpi::PhysicalPosition::new(
            center_x as f64,
            center_y as f64,
        ))
        .is_ok()
    {
        world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .mouse
            .position = nalgebra_glm::vec2(center_x, center_y);
        world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .mouse
            .position_delta = nalgebra_glm::vec2(0.0, 0.0);
    }
}

pub(crate) fn input_event_system(world: &mut World, window_event: &WindowEvent) {
    let _span = tracing::info_span!("input_event").entered();
    use crate::ecs::event_bus::commands::publish_event;
    use crate::ecs::world::events::{InputEvent, KeyState, Message};
    use crate::ecs::world::{Vec2, resources::MouseState};

    if let winit::event::WindowEvent::KeyboardInput {
        event:
            winit::event::KeyEvent {
                physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                state,
                text,
                ..
            },
        ..
    } = window_event
    {
        let was_pressed = world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .keyboard
            .keystates
            .get(key_code)
            .is_some_and(|previous| *previous == winit::event::ElementState::Pressed);

        *world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .keyboard
            .keystates
            .entry(*key_code)
            .or_insert(*state) = *state;

        let pressed = *state == winit::event::ElementState::Pressed;
        world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .keyboard
            .frame_keys
            .push((*key_code, pressed));

        if pressed && !was_pressed {
            world
                .expect_resource_mut::<crate::ecs::input::resources::Input>()
                .keyboard
                .just_pressed_keys
                .insert(*key_code);
        } else if !pressed && was_pressed {
            world
                .expect_resource_mut::<crate::ecs::input::resources::Input>()
                .keyboard
                .just_released_keys
                .insert(*key_code);
        }

        if pressed && let Some(text) = text {
            for character in text.chars() {
                world
                    .expect_resource_mut::<crate::ecs::input::resources::Input>()
                    .keyboard
                    .frame_chars
                    .push(character);
            }
        }

        let event_state = match state {
            winit::event::ElementState::Pressed => InputEvent::KeyboardInput {
                key_code: *key_code as u32,
                state: KeyState::Pressed,
            },
            winit::event::ElementState::Released => InputEvent::KeyboardInput {
                key_code: *key_code as u32,
                state: KeyState::Released,
            },
        };

        publish_event(world, Message::Input { event: event_state });
    }

    if let winit::event::WindowEvent::Ime(ime_event) = window_event {
        let ime = &mut world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .keyboard
            .ime;
        match ime_event {
            winit::event::Ime::Enabled => {
                ime.preedit = None;
            }
            winit::event::Ime::Preedit(text, cursor) => {
                ime.preedit = Some((text.clone(), *cursor));
            }
            winit::event::Ime::Commit(text) => {
                ime.committed_this_frame.push(text.clone());
                ime.preedit = None;
            }
            winit::event::Ime::Disabled => {
                ime.preedit = None;
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    let cursor_scale = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .handle
        .as_ref()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);
    #[cfg(not(target_arch = "wasm32"))]
    let cursor_scale = 1.0_f32;

    let pointer_scale = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .pointer_viewport_scale;

    let mouse = &mut world
        .expect_resource_mut::<crate::ecs::input::resources::Input>()
        .mouse;
    match window_event {
        winit::event::WindowEvent::MouseInput { button, state, .. } => {
            let pressed = *state == winit::event::ElementState::Pressed;
            let released = *state == winit::event::ElementState::Released;
            match button {
                winit::event::MouseButton::Left => {
                    let was_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
                    mouse.state.set(MouseState::LEFT_CLICKED, pressed);
                    if pressed && !was_clicked {
                        mouse.state.set(MouseState::LEFT_JUST_PRESSED, true);
                    }
                    if released && was_clicked {
                        mouse.state.set(MouseState::LEFT_JUST_RELEASED, true);
                    }
                }
                winit::event::MouseButton::Middle => {
                    let was_clicked = mouse.state.contains(MouseState::MIDDLE_CLICKED);
                    mouse.state.set(MouseState::MIDDLE_CLICKED, pressed);
                    if pressed && !was_clicked {
                        mouse.state.set(MouseState::MIDDLE_JUST_PRESSED, true);
                    }
                    if released && was_clicked {
                        mouse.state.set(MouseState::MIDDLE_JUST_RELEASED, true);
                    }
                }
                winit::event::MouseButton::Right => {
                    let was_clicked = mouse.state.contains(MouseState::RIGHT_CLICKED);
                    mouse.state.set(MouseState::RIGHT_CLICKED, pressed);
                    if pressed && !was_clicked {
                        mouse.state.set(MouseState::RIGHT_JUST_PRESSED, true);
                    }
                    if released && was_clicked {
                        mouse.state.set(MouseState::RIGHT_JUST_RELEASED, true);
                    }
                }
                _ => {}
            }
        }
        winit::event::WindowEvent::CursorMoved { position, .. } => {
            let current_position = Vec2::new(position.x as f32, position.y as f32) * pointer_scale;
            if mouse.position_initialized {
                mouse.position_delta += (current_position - mouse.position) * cursor_scale;
            } else {
                mouse.position_initialized = true;
            }
            mouse.position = current_position;
            mouse.state.set(MouseState::MOVED, true);
        }
        winit::event::WindowEvent::CursorLeft { .. } => {
            mouse.position_initialized = false;
        }
        winit::event::WindowEvent::MouseWheel { delta, .. } => {
            let (h_delta, v_delta) = match delta {
                winit::event::MouseScrollDelta::LineDelta(h, v) => (*h, *v),
                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                    (pos.x as f32 / 120.0, pos.y as f32 / 120.0)
                }
            };
            mouse.wheel_delta = Vec2::new(h_delta, v_delta);
            mouse.state.set(MouseState::SCROLLED, true);
        }
        _ => {}
    }

    handle_touch_input(world, window_event);
}

fn handle_touch_input(world: &mut World, window_event: &WindowEvent) {
    use crate::ecs::input::resources::{TouchPhase, TouchPoint};
    use crate::ecs::world::Vec2;

    if let winit::event::WindowEvent::Touch(touch) = window_event {
        let pointer_scale = world
            .expect_resource_mut::<crate::ecs::window::resources::Window>()
            .pointer_viewport_scale;
        let touch_input = &mut world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .touch;
        let id = touch.id;
        let position = Vec2::new(touch.location.x as f32, touch.location.y as f32) * pointer_scale;

        let was_primary = touch_input.primary_touch_id == Some(id);

        match touch.phase {
            winit::event::TouchPhase::Started => {
                let touch_point = TouchPoint {
                    id,
                    position,
                    start_position: position,
                    previous_position: position,
                    phase: TouchPhase::Started,
                };
                touch_input.touches.insert(id, touch_point);

                if touch_input.primary_touch_id.is_none() {
                    touch_input.primary_touch_id = Some(id);
                } else if touch_input.secondary_touch_id.is_none() {
                    touch_input.secondary_touch_id = Some(id);
                }
            }
            winit::event::TouchPhase::Moved => {
                if let Some(touch_point) = touch_input.touches.get_mut(&id) {
                    touch_point.previous_position = touch_point.position;
                    touch_point.position = position;
                    touch_point.phase = TouchPhase::Moved;
                }
            }
            winit::event::TouchPhase::Ended => {
                if let Some(touch_point) = touch_input.touches.get_mut(&id) {
                    touch_point.previous_position = touch_point.position;
                    touch_point.position = position;
                    touch_point.phase = TouchPhase::Ended;
                }
            }
            winit::event::TouchPhase::Cancelled => {
                if let Some(touch_point) = touch_input.touches.get_mut(&id) {
                    touch_point.phase = TouchPhase::Cancelled;
                }
            }
        }

        touch_input.update_gesture();

        let is_primary = touch_input.primary_touch_id == Some(id);
        let mouse = &mut world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .mouse;
        match touch.phase {
            winit::event::TouchPhase::Started if is_primary => {
                if mouse.position_initialized {
                    mouse.position_delta += position - mouse.position;
                } else {
                    mouse.position_initialized = true;
                }
                mouse.position = position;
                mouse.state.set(MouseState::MOVED, true);
                let was_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
                mouse.state.set(MouseState::LEFT_CLICKED, true);
                if !was_clicked {
                    mouse.state.set(MouseState::LEFT_JUST_PRESSED, true);
                }
            }
            winit::event::TouchPhase::Moved if is_primary => {
                if mouse.position_initialized {
                    mouse.position_delta += position - mouse.position;
                }
                mouse.position = position;
                mouse.state.set(MouseState::MOVED, true);
            }
            winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled
                if was_primary =>
            {
                mouse.position = position;
                let was_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
                mouse.state.set(MouseState::LEFT_CLICKED, false);
                if was_clicked {
                    mouse.state.set(MouseState::LEFT_JUST_RELEASED, true);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn forward_window_events(world: &mut World, event: &WindowEvent) {
    use crate::ecs::input::events::AppEvent;

    match event {
        WindowEvent::DroppedFile(path) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if path.is_dir() {
                    let mut files = Vec::new();
                    super::file_drop::collect_directory_files(path, path, &mut files);
                    if files.is_empty() {
                        world
                            .expect_resource_mut::<crate::ecs::input::resources::Input>()
                            .events
                            .push(AppEvent::FileDroppedPath(path.clone()));
                    } else {
                        for file in files {
                            world
                                .expect_resource_mut::<crate::ecs::input::resources::Input>()
                                .events
                                .push(AppEvent::FileDropped(file));
                        }
                    }
                } else {
                    world
                        .expect_resource_mut::<crate::ecs::input::resources::Input>()
                        .events
                        .push(AppEvent::FileDroppedPath(path.clone()));
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                world
                    .expect_resource_mut::<crate::ecs::input::resources::Input>()
                    .events
                    .push(AppEvent::FileDroppedPath(path.clone()));
            }
        }
        WindowEvent::HoveredFile(path) => {
            world
                .expect_resource_mut::<crate::ecs::input::resources::Input>()
                .events
                .push(AppEvent::FileHovered(path.clone()));
        }
        WindowEvent::HoveredFileCancelled => {
            world
                .expect_resource_mut::<crate::ecs::input::resources::Input>()
                .events
                .push(AppEvent::FileHoverCancelled);
        }
        _ => {}
    }

    if world.resources.user_interface.consumed_event {
        return;
    }

    if let WindowEvent::KeyboardInput {
        event:
            winit::event::KeyEvent {
                physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                state: key_state,
                ..
            },
        ..
    } = event
    {
        world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .events
            .push(AppEvent::Keyboard {
                key: *key_code,
                state: *key_state,
            });
    }

    if let WindowEvent::MouseInput {
        button,
        state: mouse_state,
        ..
    } = event
    {
        world
            .expect_resource_mut::<crate::ecs::input::resources::Input>()
            .events
            .push(AppEvent::Mouse {
                button: *button,
                state: *mouse_state,
            });
    }
}
