//! Headless and offscreen frame driver.
//!
//! The winit event loop in [`crate::run`] owns input, timing, and the render
//! loop. That path assumes a real OS window. This module exposes the same frame
//! pipeline without winit, so the engine can render into a surface that has no
//! window behind it:
//!
//! - a `web_sys::OffscreenCanvas` transferred to a web worker, where the render
//!   loop runs off the browser's main thread, or
//! - any native offscreen target for golden-image tests, thumbnailing, or
//!   server-side capture.
//!
//! # Lifecycle
//!
//! Build a renderer from the surface target with
//! [`create_wgpu_renderer`](crate::render::wgpu::create_wgpu_renderer) (its only
//! bound is `Into<wgpu::SurfaceTarget>`, which an `OffscreenCanvas` satisfies),
//! then:
//!
//! ```ignore
//! let mut renderer = create_wgpu_renderer(offscreen_canvas, width, height).await?;
//! initialize_offscreen(&mut world, &mut state, &mut renderer, (width, height), 1.0);
//!
//! // once per requestAnimationFrame tick:
//! if let Some(next_state) = tick_offscreen(&mut world, &mut state, &mut renderer) {
//!     state = next_state;
//! }
//!
//! // on a surface resize:
//! resize_offscreen(&mut world, &mut renderer, new_width, new_height);
//! ```
//!
//! Forwarded pointer, wheel, and keyboard events are fed in with the
//! `input_inject_*` functions, which write the same input state the winit loop
//! produces.

use crate::ecs::world::Vec2;
use crate::ecs::world::World;
use crate::platform::input::resources::MouseState;
use crate::platform::input::resources::{TouchPhase, TouchPoint};
use crate::render::wgpu::WgpuRenderer;
use crate::state::State;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

/// Prepares the world for offscreen rendering and runs [`State::initialize`].
///
/// Sets the cached viewport size and scale factor the render passes and UI
/// layout read each frame, configures the render graph from the state, then
/// initializes the scene. Call once after the renderer is created.
pub fn initialize_offscreen(
    world: &mut World,
    state: &mut dyn State,
    renderer: &mut WgpuRenderer,
    viewport: (u32, u32),
    scale_factor: f32,
) {
    world
        .res_mut::<crate::platform::window::Window>()
        .cached_viewport_size = Some(viewport);
    world
        .res_mut::<crate::platform::window::Window>()
        .cached_scale_factor = scale_factor;

    if let Err(error) = crate::render_driver::configure_with_state(renderer, state) {
        tracing::error!("Failed to configure renderer with state: {error}");
    }

    state.initialize(world);
}

/// Resizes the surface and updates the cached viewport size used by the render
/// passes and UI layout.
pub fn resize_offscreen(world: &mut World, renderer: &mut WgpuRenderer, width: u32, height: u32) {
    if let Err(error) = renderer.resize_surface(width, height) {
        tracing::error!("Failed to resize surface: {error}");
    }
    world
        .res_mut::<crate::platform::window::Window>()
        .cached_viewport_size = Some((width, height));
}

/// Advances and renders one offscreen frame.
///
/// Advances timing, then runs the shared frame body
/// ([`run_frame_body`](crate::run::run_frame_body)): [`State::run_systems`]
/// (the per-frame stages, engine systems included) and the render graph.
/// Returns the next [`State`] when one was requested through
/// `world.res::<crate::state::PendingState>()`.
pub fn tick_offscreen(
    world: &mut World,
    state: &mut dyn State,
    renderer: &mut WgpuRenderer,
) -> Option<Box<dyn State>> {
    super::advance_timing(world);
    super::run_frame_body(world, state, renderer)
}

/// Injects an absolute cursor position (in physical pixels), updating the
/// per-frame position delta the same way the winit loop does.
pub fn input_inject_cursor_moved(world: &mut World, position: Vec2) {
    let mouse = &mut world
        .res_mut::<crate::platform::input::resources::Input>()
        .mouse;
    if mouse.position_initialized {
        mouse.position_delta += position - mouse.position;
    } else {
        mouse.position_initialized = true;
    }
    mouse.position = position;
    mouse.state.set(MouseState::MOVED, true);
}

/// Accumulates raw mouse motion for first-person look, matching the winit
/// device-event path.
pub fn input_inject_mouse_motion(world: &mut World, delta: Vec2) {
    world
        .res_mut::<crate::platform::input::resources::Input>()
        .mouse
        .raw_mouse_delta += delta;
}

/// Injects a mouse button press or release, maintaining the just-pressed and
/// just-released edge flags.
pub fn input_inject_mouse_button(world: &mut World, button: MouseButton, state: ElementState) {
    let pressed = state == ElementState::Pressed;
    let released = state == ElementState::Released;
    let mouse = &mut world
        .res_mut::<crate::platform::input::resources::Input>()
        .mouse;
    match button {
        MouseButton::Left => {
            let was_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
            mouse.state.set(MouseState::LEFT_CLICKED, pressed);
            if pressed && !was_clicked {
                mouse.state.set(MouseState::LEFT_JUST_PRESSED, true);
            }
            if released && was_clicked {
                mouse.state.set(MouseState::LEFT_JUST_RELEASED, true);
            }
        }
        MouseButton::Middle => {
            let was_clicked = mouse.state.contains(MouseState::MIDDLE_CLICKED);
            mouse.state.set(MouseState::MIDDLE_CLICKED, pressed);
            if pressed && !was_clicked {
                mouse.state.set(MouseState::MIDDLE_JUST_PRESSED, true);
            }
            if released && was_clicked {
                mouse.state.set(MouseState::MIDDLE_JUST_RELEASED, true);
            }
        }
        MouseButton::Right => {
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

/// Injects a mouse wheel delta in lines.
pub fn input_inject_mouse_wheel(world: &mut World, delta: Vec2) {
    let mouse = &mut world
        .res_mut::<crate::platform::input::resources::Input>()
        .mouse;
    mouse.wheel_delta = delta;
    mouse.state.set(MouseState::SCROLLED, true);
}

/// Injects a touch contact in window coordinates (physical pixels), updating
/// the multi-touch state and recomputing the current gesture the same way the
/// winit loop does. This drives the touch pan-orbit controller (single-finger
/// orbit, two-finger pan, pinch zoom). Mouse emulation, if a caller also wants
/// pointer-style picking from touch, is left to the caller.
pub fn input_inject_touch(world: &mut World, id: u64, phase: TouchPhase, position: Vec2) {
    let touch = &mut world
        .res_mut::<crate::platform::input::resources::Input>()
        .touch;
    match phase {
        TouchPhase::Started => {
            touch.touches.insert(
                id,
                TouchPoint {
                    id,
                    position,
                    start_position: position,
                    previous_position: position,
                    phase: TouchPhase::Started,
                },
            );
            if touch.primary_touch_id.is_none() {
                touch.primary_touch_id = Some(id);
            } else if touch.secondary_touch_id.is_none() {
                touch.secondary_touch_id = Some(id);
            }
        }
        TouchPhase::Moved => {
            if let Some(touch_point) = touch.touches.get_mut(&id) {
                touch_point.previous_position = touch_point.position;
                touch_point.position = position;
                touch_point.phase = TouchPhase::Moved;
            }
        }
        TouchPhase::Ended => {
            if let Some(touch_point) = touch.touches.get_mut(&id) {
                touch_point.previous_position = touch_point.position;
                touch_point.position = position;
                touch_point.phase = TouchPhase::Ended;
            }
        }
        TouchPhase::Cancelled => {
            if let Some(touch_point) = touch.touches.get_mut(&id) {
                touch_point.phase = TouchPhase::Cancelled;
            }
        }
    }
    touch.update_gesture();
}

/// Injects a keyboard key state change with optional typed text, updating the
/// per-key state map, the just-pressed and just-released sets, the per-frame key
/// and character buffers, and the input event bus.
pub fn input_inject_keyboard(
    world: &mut World,
    key_code: KeyCode,
    state: ElementState,
    text: Option<&str>,
) {
    use crate::ecs::bootstrap::events::{InputEvent, KeyState, Message};
    use crate::ecs::event_bus::commands::publish_event;

    let was_pressed = world
        .res::<crate::platform::input::resources::Input>()
        .keyboard
        .keystates
        .get(&key_code)
        .is_some_and(|previous| *previous == ElementState::Pressed);

    *world
        .res_mut::<crate::platform::input::resources::Input>()
        .keyboard
        .keystates
        .entry(key_code)
        .or_insert(state) = state;

    let pressed = state == ElementState::Pressed;
    world
        .res_mut::<crate::platform::input::resources::Input>()
        .keyboard
        .frame_keys
        .push((key_code, pressed));

    if pressed && !was_pressed {
        world
            .res_mut::<crate::platform::input::resources::Input>()
            .keyboard
            .just_pressed_keys
            .insert(key_code);
    } else if !pressed && was_pressed {
        world
            .res_mut::<crate::platform::input::resources::Input>()
            .keyboard
            .just_released_keys
            .insert(key_code);
    }

    if pressed && let Some(text) = text {
        for character in text.chars() {
            world
                .res_mut::<crate::platform::input::resources::Input>()
                .keyboard
                .frame_chars
                .push(character);
        }
    }

    let event = match state {
        ElementState::Pressed => InputEvent::KeyboardInput {
            key_code: key_code as u32,
            state: KeyState::Pressed,
        },
        ElementState::Released => InputEvent::KeyboardInput {
            key_code: key_code as u32,
            state: KeyState::Released,
        },
    };
    publish_event(world, Message::Input { event });
}

/// Drives a `requestAnimationFrame` render loop on the worker's
/// `DedicatedWorkerGlobalScope`, invoking `on_frame` once per frame.
///
/// Web workers that render an `OffscreenCanvas` otherwise hand-write the
/// self-rescheduling closure that this owns. The caller's `on_frame` does the
/// per-frame work, typically locking its shared app state and calling
/// [`tick_offscreen`]. The page-side glue (canvas transfer and message
/// decoding) stays app-specific and is not covered here.
#[cfg(target_arch = "wasm32")]
type AnimationFrame =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut()>>>>;

#[cfg(target_arch = "wasm32")]
pub fn spawn_animation_frame_loop(mut on_frame: impl FnMut() + 'static) {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;

    let scope: web_sys::DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let frame: AnimationFrame = Rc::new(RefCell::new(None));
    let frame_handle = frame.clone();
    let loop_scope = scope.clone();

    *frame.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        on_frame();
        if let Some(callback) = frame_handle.borrow().as_ref() {
            let _ = loop_scope.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }));

    if let Some(callback) = frame.borrow().as_ref() {
        let _ = scope.request_animation_frame(callback.as_ref().unchecked_ref());
    }
}
