use crate::host::Host;
use crate::renderer::Renderer;
use nalgebra_glm::Vec2;
use nightshade::World;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::contract::input::{TouchPhase, TouchPoint};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

pub fn initialize_offscreen<R: Renderer>(
    world: &mut World,
    host: &mut dyn Host,
    renderer: &mut R,
    viewport: (u32, u32),
    scale_factor: f32,
) {
    let window = world
        .resources
        .res_mut::<nightshade_platform::contract::window::Window>();
    window.cached_viewport_size = Some(viewport);
    window.cached_scale_factor = scale_factor;

    if let Err(error) = renderer.configure() {
        panic!("the renderer could not configure the surface: {error}");
    }

    host.initialize(world);
}

pub fn resize_offscreen<R: Renderer>(
    resources: &mut nightshade::ecs::ResourceMap,
    renderer: &mut R,
    width: u32,
    height: u32,
) {
    if let Err(error) = renderer.resize(width, height) {
        tracing::error!("Failed to resize surface: {error}");
    }
    resources
        .res_mut::<nightshade_platform::contract::window::Window>()
        .cached_viewport_size = Some((width, height));
}

pub fn tick_offscreen<R: Renderer>(
    world: &mut World,
    host: &mut dyn Host,
    renderer: &mut R,
) -> Option<Box<dyn Host>> {
    super::advance_timing(&mut world.resources);
    super::run_frame_body(world, host, renderer)
}

pub fn input_inject_cursor_moved(resources: &mut nightshade::ecs::ResourceMap, position: Vec2) {
    let mouse = &mut resources
        .res_mut::<nightshade_platform::contract::input::Input>()
        .mouse;
    if mouse.position_initialized {
        mouse.position_delta += position - mouse.position;
    } else {
        mouse.position_initialized = true;
    }
    mouse.position = position;
    mouse.state.set(MouseState::MOVED, true);
}

pub fn input_inject_mouse_motion(resources: &mut nightshade::ecs::ResourceMap, delta: Vec2) {
    resources
        .res_mut::<nightshade_platform::contract::input::Input>()
        .mouse
        .raw_mouse_delta += delta;
}

pub fn input_inject_mouse_button(
    resources: &mut nightshade::ecs::ResourceMap,
    button: MouseButton,
    state: ElementState,
) {
    let pressed = state == ElementState::Pressed;
    let released = state == ElementState::Released;
    let mouse = &mut resources
        .res_mut::<nightshade_platform::contract::input::Input>()
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

pub fn input_inject_mouse_wheel(resources: &mut nightshade::ecs::ResourceMap, delta: Vec2) {
    let mouse = &mut resources
        .res_mut::<nightshade_platform::contract::input::Input>()
        .mouse;
    mouse.wheel_delta = delta;
    mouse.state.set(MouseState::SCROLLED, true);
}

pub fn input_inject_touch(
    resources: &mut nightshade::ecs::ResourceMap,
    id: u64,
    phase: TouchPhase,
    position: Vec2,
) {
    let touch = &mut resources
        .res_mut::<nightshade_platform::contract::input::Input>()
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

pub fn input_inject_keyboard(
    resources: &mut nightshade::ecs::ResourceMap,
    key_code: KeyCode,
    state: ElementState,
    text: Option<&str>,
) {
    let was_pressed = resources
        .res::<nightshade_platform::contract::input::Input>()
        .keyboard
        .keystates
        .get(&key_code)
        .is_some_and(|previous| *previous == ElementState::Pressed);

    *resources
        .res_mut::<nightshade_platform::contract::input::Input>()
        .keyboard
        .keystates
        .entry(key_code)
        .or_insert(state) = state;

    let pressed = state == ElementState::Pressed;
    resources
        .res_mut::<nightshade_platform::contract::input::Input>()
        .keyboard
        .frame_keys
        .push((key_code, pressed));

    if pressed && !was_pressed {
        resources
            .res_mut::<nightshade_platform::contract::input::Input>()
            .keyboard
            .just_pressed_keys
            .insert(key_code);
    } else if !pressed && was_pressed {
        resources
            .res_mut::<nightshade_platform::contract::input::Input>()
            .keyboard
            .just_released_keys
            .insert(key_code);
    }

    if pressed && let Some(text) = text {
        for character in text.chars() {
            resources
                .res_mut::<nightshade_platform::contract::input::Input>()
                .keyboard
                .frame_chars
                .push(character);
        }
    }
}

#[cfg(target_arch = "wasm32")]
type FrameWork = std::rc::Rc<std::cell::RefCell<Option<Box<dyn FnMut(f64)>>>>;

#[cfg(target_arch = "wasm32")]
type LoopCallback<Args> =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut(Args)>>>>;

#[cfg(target_arch = "wasm32")]
type TimerCallback =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut()>>>>;

#[cfg(target_arch = "wasm32")]
pub fn spawn_animation_frame_loop(on_frame: impl FnMut(f64) + 'static) {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;

    const TIMER_INTERVAL_MS: i32 = 16;

    #[derive(Clone, Copy, PartialEq)]
    enum Driver {
        AnimationFrame,
        Timer,
    }

    let scope: web_sys::DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let driver = Rc::new(Cell::new(Driver::AnimationFrame));
    let work: FrameWork = Rc::new(RefCell::new(Some(Box::new(on_frame))));

    let animation_frame: LoopCallback<f64> = Rc::new(RefCell::new(None));
    let timer: TimerCallback = Rc::new(RefCell::new(None));

    fn run_frame(work: &FrameWork, timestamp: f64) {
        let taken = work.borrow_mut().take();
        if let Some(mut callback) = taken {
            callback(timestamp);
            *work.borrow_mut() = Some(callback);
        }
    }

    fn request_timer(scope: &web_sys::DedicatedWorkerGlobalScope, callback: &TimerCallback) {
        let Some(callback) = callback.borrow().as_ref().map(|callback| {
            let handle: &wasm_bindgen::JsValue = callback.as_ref();
            handle.clone()
        }) else {
            return;
        };
        if let Err(error) = scope.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            TIMER_INTERVAL_MS,
        ) {
            tracing::error!("the worker render loop has no timer either: {error:?}");
        }
    }

    {
        let scope = scope.clone();
        let timer_handle = timer.clone();
        let driver_handle = driver.clone();
        let work = work.clone();
        *timer.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
            if driver_handle.get() != Driver::Timer {
                return;
            }
            request_timer(&scope, &timer_handle);
            run_frame(&work, js_sys::Date::now());
        }));
    }

    {
        let scope = scope.clone();
        let animation_frame_handle = animation_frame.clone();
        let timer_handle = timer.clone();
        let driver_handle = driver.clone();
        let work = work.clone();
        *animation_frame.borrow_mut() =
            Some(Closure::<dyn FnMut(f64)>::new(move |timestamp: f64| {
                driver_handle.set(Driver::AnimationFrame);
                if let Some(callback) = animation_frame_handle.borrow().as_ref()
                    && let Err(error) =
                        scope.request_animation_frame(callback.as_ref().unchecked_ref())
                {
                    tracing::error!("the worker render loop lost its animation frame: {error:?}");
                    driver_handle.set(Driver::Timer);
                    request_timer(&scope, &timer_handle);
                }
                run_frame(&work, timestamp);
            }));
    }

    let requested = animation_frame
        .borrow()
        .as_ref()
        .map(|callback| scope.request_animation_frame(callback.as_ref().unchecked_ref()));

    if !matches!(requested, Some(Ok(_))) {
        tracing::warn!("this worker has no animation frames; the render loop uses a timer");
        driver.set(Driver::Timer);
        request_timer(&scope, &timer);
    }

    std::mem::forget(animation_frame);
    std::mem::forget(timer);
}
