use nalgebra_glm::Vec2;
use nightshade_platform::commands::input::{apply_key, apply_touch};
use nightshade_platform::contract::input::{Input, MouseState, TouchPhase};
use nightshade_platform::contract::window::Window;
use winit::event::WindowEvent;

pub fn apply_window_event(
    resources: &mut nightshade::ecs::ResourceMap,
    window_event: &WindowEvent,
) {
    if let winit::event::WindowEvent::KeyboardInput {
        event:
            winit::event::KeyEvent {
                physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                state,
                text,
                repeat,
                ..
            },
        ..
    } = window_event
    {
        apply_key(
            resources.res_mut::<Input>(),
            *key_code,
            *state,
            *repeat,
            text.as_deref(),
        );
    }

    if let winit::event::WindowEvent::Ime(ime_event) = window_event {
        let ime = &mut resources.res_mut::<Input>().keyboard.ime;
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
    let cursor_scale = resources
        .res::<Window>()
        .handle
        .as_ref()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);
    #[cfg(not(target_arch = "wasm32"))]
    let cursor_scale = 1.0_f32;

    let pointer_scale = resources.res::<Window>().pointer_viewport_scale;

    let mouse = &mut resources.res_mut::<Input>().mouse;
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

    handle_touch_input(resources, window_event);
}

fn handle_touch_input(resources: &mut nightshade::ecs::ResourceMap, window_event: &WindowEvent) {
    let winit::event::WindowEvent::Touch(touch) = window_event else {
        return;
    };
    let pointer_scale = resources.res::<Window>().pointer_viewport_scale;
    let position = Vec2::new(touch.location.x as f32, touch.location.y as f32) * pointer_scale;
    let phase = match touch.phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    };
    apply_touch(resources.res_mut::<Input>(), touch.id, position, phase);
}
