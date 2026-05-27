//! Application entry point and main loop.
//!
//! Launch a Nightshade application by implementing [`State`] and calling [`launch`]:
//!
//! ```ignore
//! struct MyGame;
//!
//! impl State for MyGame {
//!     fn initialize(&mut self, world: &mut World) {
//!         world.resources.window.title = "My Game".to_string();
//!         // Set up camera, lights, entities
//!     }
//!
//!     fn run_systems(&mut self, world: &mut World) {
//!         // Per-frame game logic
//!     }
//! }
//!
//! fn main() {
//!     nightshade::launch(MyGame).unwrap();
//! }
//! ```
//!
//! # State Trait Methods
//!
//! The [`State`] trait provides hooks into the engine lifecycle:
//!
//! | Method | When Called | Use For |
//! |--------|-------------|---------|
//! | `initialize()` | After renderer ready | Scene setup, spawn entities |
//! | `run_systems()` | Every frame | Game logic, drain `world.resources.input.events` |
//! | `configure_render_graph()` | Once at startup | Custom post-processing |
//! | `update_render_graph()` | Every frame | Dynamic render graph changes |
//! | `pre_render()` | Each frame before render | Custom GPU uploads |
//!
//! Input, file-drop, gamepad, and lifecycle events arrive on
//! `world.resources.input.events` as [`AppEvent`](crate::ecs::input::events::AppEvent)
//! values. Drain them in [`State::run_systems`].
//!
//! To transition to a different `State`, set
//! `world.resources.window.next_state = Some(Box::new(|world| Box::new(NewState::new(world))))`
//! from inside `run_systems`; the engine takes the builder at the end of the
//! frame and invokes it with `&mut World` so the new state can be constructed
//! against the live world.
//!
//! # Typical Initialize Pattern
//!
//! ```ignore
//! fn initialize(&mut self, world: &mut World) {
//!     // Configure graphics
//!     world.resources.user_interface.enabled = true;
//!     world.resources.render_settings.atmosphere = Atmosphere::Sky;
//!     world.resources.debug_draw.show_grid = true;
//!
//!     // Spawn camera
//!     let camera = spawn_camera(world, Vec3::new(5.0, 3.0, 5.0), "Camera".to_string());
//!     world.resources.active_camera = Some(camera);
//!
//!     // Spawn lighting
//!     spawn_sun(world);
//!
//!     // Spawn entities
//!     let cube = spawn_cube_at(world, Vec3::new(0.0, 1.0, 0.0));
//! }
//! ```
//!
//! # State Transitions
//!
//! Switch between game states (menu, gameplay, pause) by writing a builder
//! closure to the `next_state` field of the
//! [`Window`](crate::ecs::window::resources::Window) resource from inside
//! [`State::run_systems`]. The closure receives `&mut World` so the new state
//! can be constructed against live engine data.
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     if self.should_go_to_menu {
//!         world.resources.window.next_state =
//!             Some(Box::new(|_world| Box::new(MenuState::default())));
//!     }
//! }
//! ```

use crate::prelude::*;

use crate::ecs::world::World;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
pub use crate::logging::log_file_name;
pub use crate::logging::{LogConfig, LogRotation};
pub use crate::state::{NextStateBuilder, RenderResources, State};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_DROPPED_FILES: std::cell::RefCell<Vec<crate::ecs::input::resources::DroppedFile>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Launches the application with the given state.
pub fn launch(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    launch_windowed(state)
}

/// Launches a windowed application (desktop or web).
pub fn launch_windowed(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    #[allow(unused_mut)]
    let mut world = World::default();

    #[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
    let (log_config, log_file_name) = {
        let config = world.resources.window.log_config.clone();
        let file_name = log_file_name(&world.resources.window.title, &config);
        (config, file_name)
    };

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "tracing")))]
    tracing_subscriber::fmt::init();

    #[cfg(all(
        not(target_arch = "wasm32"),
        feature = "tracing",
        not(feature = "tracy"),
        not(feature = "chrome")
    ))]
    let _guard = {
        use tracing_subscriber::EnvFilter;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let file_appender = match log_config.rotation {
            LogRotation::Daily => {
                tracing_appender::rolling::daily(&log_config.directory, &log_file_name)
            }
            _ => tracing_appender::rolling::never(&log_config.directory, &log_file_name),
        };
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&log_config.default_filter));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false),
            )
            .init();

        guard
    };

    #[cfg(all(
        not(target_arch = "wasm32"),
        feature = "tracy",
        not(feature = "chrome")
    ))]
    let _guard = {
        use tracing_subscriber::EnvFilter;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let file_appender = match log_config.rotation {
            LogRotation::Daily => {
                tracing_appender::rolling::daily(&log_config.directory, &log_file_name)
            }
            _ => tracing_appender::rolling::never(&log_config.directory, &log_file_name),
        };
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&log_config.default_filter));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false),
            )
            .with(tracing_tracy::TracyLayer::default())
            .init();

        guard
    };

    #[cfg(all(
        not(target_arch = "wasm32"),
        feature = "chrome",
        not(feature = "tracy")
    ))]
    let _guard = {
        use tracing_subscriber::EnvFilter;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let file_appender = match log_config.rotation {
            LogRotation::Daily => {
                tracing_appender::rolling::daily(&log_config.directory, &log_file_name)
            }
            _ => tracing_appender::rolling::never(&log_config.directory, &log_file_name),
        };
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&log_config.default_filter));

        let (chrome_layer, chrome_guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file("logs/trace.json")
            .build();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false),
            )
            .with(chrome_layer)
            .init();

        (file_guard, chrome_guard)
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "tracy", feature = "chrome"))]
    let _guard = {
        use tracing_subscriber::EnvFilter;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let file_appender = match log_config.rotation {
            LogRotation::Daily => {
                tracing_appender::rolling::daily(&log_config.directory, &log_file_name)
            }
            _ => tracing_appender::rolling::never(&log_config.directory, &log_file_name),
        };
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&log_config.default_filter));

        let (chrome_layer, chrome_guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file("logs/trace.json")
            .build();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false),
            )
            .with(tracing_tracy::TracyLayer::default())
            .with(chrome_layer)
            .init();

        (file_guard, chrome_guard)
    };

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        tracing_subscriber::fmt()
            .with_writer(tracing_web::MakeConsoleWriter)
            .without_time()
            .init();
    }

    let state = Box::new(state);

    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut context = WindowContext {
            state,
            world,
            renderer: None,
            snapshot_path: None,
            snapshot_queued: false,
            initialized: false,
        };
        event_loop.run_app(&mut context)?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        let context = WindowContext {
            state,
            world,
            renderer: None,
            renderer_receiver: None,
            initialized: false,
        };
        event_loop.spawn_app(context);
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_min_window_size(world: &mut World) {
    let desired = world.resources.renderer_state.min_window_size;
    if world.resources.window.applied_min_window_size == desired {
        return;
    }
    let Some(handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    match desired {
        Some((min_width, min_height)) => {
            handle.set_min_inner_size(Some(winit::dpi::LogicalSize::new(
                min_width as f64,
                min_height as f64,
            )));
            let scale = handle.scale_factor();
            let physical_min_width = (min_width as f64 * scale).round() as u32;
            let physical_min_height = (min_height as f64 * scale).round() as u32;
            let current = handle.inner_size();
            if current.width < physical_min_width || current.height < physical_min_height {
                let _ = handle.request_inner_size(winit::dpi::PhysicalSize::new(
                    current.width.max(physical_min_width),
                    current.height.max(physical_min_height),
                ));
            }
        }
        None => {
            handle.set_min_inner_size(None::<winit::dpi::LogicalSize<f64>>);
        }
    }
    world.resources.window.applied_min_window_size = desired;
}

#[cfg(target_arch = "wasm32")]
fn sync_min_window_size(_world: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
fn sync_window_title(world: &mut World) {
    if world.resources.window.applied_title.as_deref() == Some(&world.resources.window.title) {
        return;
    }
    let Some(handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    handle.set_title(&world.resources.window.title);
    world.resources.window.applied_title = Some(world.resources.window.title.clone());
}

#[cfg(target_arch = "wasm32")]
fn sync_window_title(_world: &mut World) {}

#[cfg(all(not(target_arch = "wasm32"), feature = "assets"))]
fn sync_window_icon(world: &mut World) {
    let current = world.resources.window.icon_bytes;
    let applied = world.resources.window.applied_icon_bytes;
    let unchanged = match (current, applied) {
        (None, None) => true,
        (Some(left), Some(right)) => std::ptr::eq(left.as_ptr(), right.as_ptr()),
        _ => false,
    };
    if unchanged {
        return;
    }
    let Some(handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    let icon = current.and_then(|bytes| {
        let image = image::load_from_memory(bytes).ok()?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        winit::window::Icon::from_rgba(rgba.into_raw(), width, height).ok()
    });
    handle.set_window_icon(icon);
    world.resources.window.applied_icon_bytes = current;
}

#[cfg(any(target_arch = "wasm32", not(feature = "assets")))]
fn sync_window_icon(_world: &mut World) {}

pub(crate) fn step(
    world: &mut World,
    state: &mut Box<dyn State + 'static>,
    renderer: &mut Option<crate::render::wgpu::WgpuRenderer>,
    event: &WindowEvent,
) -> Option<Box<dyn State>> {
    let delta_ms = (world.resources.window.timing.delta_time * 1000.0) as u32;
    let _span = tracing::info_span!("frame", delta_ms).entered();

    sync_min_window_size(world);
    sync_window_title(world);
    sync_window_icon(world);
    if matches!(event, WindowEvent::RedrawRequested) {
        throttle_to_frame_rate_limit(world);
    }
    handle_event_systems(world, event);

    match event {
        WindowEvent::RedrawRequested => {
            world.core.increment_tick();

            #[cfg(feature = "gamepad")]
            crate::ecs::input::systems::gamepad_input_system(world);
            process_dropped_files(world);

            if let Some(renderer) = renderer.as_mut() {
                renderer.initialize_fonts(world);
            }

            #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
            crate::steam::steam_run_callbacks(&world.resources.steam);

            #[cfg(feature = "gamepad")]
            {
                let gamepad_events = world.resources.input.gamepad.events.clone();
                for event in gamepad_events {
                    world
                        .resources
                        .input
                        .events
                        .push(crate::ecs::input::events::AppEvent::Gamepad(event));
                }
            }

            crate::ecs::graphics::systems::day_night_cycle_system(world);
            state.run_systems(world);
            world.resources.input.events.clear();

            run_frame_systems(world);

            if let Some(builder) = world.resources.window.next_state.take() {
                return Some(builder(world));
            }

            let window_handle = world.resources.window.handle.as_ref()?;

            if world.resources.retained_ui.enabled {
                let requested = world.resources.retained_ui.interaction.requested_cursor;
                let applied = world.resources.retained_ui.interaction.applied_cursor;
                if requested.is_some() && requested != applied {
                    if let Some(cursor) = requested {
                        window_handle.set_cursor(cursor);
                    }
                    world.resources.retained_ui.interaction.applied_cursor = requested;
                } else if requested.is_none()
                    && applied.is_some()
                    && applied != Some(winit::window::CursorIcon::Default)
                {
                    window_handle.set_cursor(winit::window::CursorIcon::Default);
                    world.resources.retained_ui.interaction.applied_cursor =
                        Some(winit::window::CursorIcon::Default);
                }
            }

            let renderer = renderer.as_mut()?;

            let should_render =
                window_handle.inner_size().width > 0 && window_handle.inner_size().height > 0;
            if should_render {
                if let Err(error) = renderer.update_with_state(state.as_mut(), world) {
                    tracing::error!("Failed to update render graph: {error}");
                }

                state.pre_render(renderer, world);

                if let Err(error) = renderer.render_frame(world) {
                    tracing::error!("Failed to draw frame: {error}");
                }
            }

            None
        }
        event => {
            forward_window_events(world, event);
            None
        }
    }
}

fn handle_event_systems(world: &mut World, event: &WindowEvent) {
    let _span = tracing::info_span!("event_handling").entered();
    input_event_system(world, event);
    timing_system(world, event);
}

pub(crate) fn run_frame_systems(world: &mut World) {
    let schedule = std::mem::take(&mut world.resources.schedules.frame);
    crate::schedule::schedule_run(&schedule, world);
    world.resources.schedules.frame = schedule;
}

fn input_event_system(world: &mut World, window_event: &WindowEvent) {
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
            .resources
            .input
            .keyboard
            .keystates
            .get(key_code)
            .is_some_and(|previous| *previous == winit::event::ElementState::Pressed);

        *world
            .resources
            .input
            .keyboard
            .keystates
            .entry(*key_code)
            .or_insert(*state) = *state;

        let pressed = *state == winit::event::ElementState::Pressed;
        world
            .resources
            .input
            .keyboard
            .frame_keys
            .push((*key_code, pressed));

        if pressed && !was_pressed {
            world
                .resources
                .input
                .keyboard
                .just_pressed_keys
                .insert(*key_code);
        } else if !pressed && was_pressed {
            world
                .resources
                .input
                .keyboard
                .just_released_keys
                .insert(*key_code);
        }

        if pressed && let Some(text) = text {
            for character in text.chars() {
                world.resources.input.keyboard.frame_chars.push(character);
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
        let ime = &mut world.resources.input.keyboard.ime;
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
        .resources
        .window
        .handle
        .as_ref()
        .map(|w| w.scale_factor() as f32)
        .unwrap_or(1.0);
    #[cfg(not(target_arch = "wasm32"))]
    let cursor_scale = 1.0_f32;

    let mouse = &mut world.resources.input.mouse;
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
            let current_position = Vec2::new(position.x as _, position.y as _);
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
        let touch_input = &mut world.resources.input.touch;
        let id = touch.id;
        let position = Vec2::new(touch.location.x as f32, touch.location.y as f32);

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
        let mouse = &mut world.resources.input.mouse;
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

/// Sleeps until the next frame slot when
/// `world.resources.render_settings.frame_rate_limit` is set. Uses the previous
/// frame's start instant from `window.timing` plus the target interval.
/// Idle-sleeps most of the gap and busy-waits the last 1ms to absorb OS
/// scheduler jitter (Windows `Sleep` resolution is ~15.6ms by default).
///
/// No-op on wasm: browsers throttle via `requestAnimationFrame` already.
#[cfg(not(target_arch = "wasm32"))]
fn throttle_to_frame_rate_limit(world: &mut World) {
    use web_time::Instant;

    let Some(target_fps) = world.resources.render_settings.frame_rate_limit else {
        return;
    };
    if !target_fps.is_finite() || target_fps <= 0.0 {
        return;
    }
    let Some(last_frame_start) = world.resources.window.timing.last_frame_start_instant else {
        return;
    };

    let target_interval = std::time::Duration::from_secs_f32(1.0 / target_fps);
    let deadline = last_frame_start + target_interval;
    let busy_wait_floor = std::time::Duration::from_millis(1);

    let now = Instant::now();
    if now >= deadline {
        return;
    }
    let remaining = deadline - now;
    if remaining > busy_wait_floor {
        std::thread::sleep(remaining - busy_wait_floor);
    }
    while Instant::now() < deadline {
        std::hint::spin_loop();
    }
}

#[cfg(target_arch = "wasm32")]
fn throttle_to_frame_rate_limit(_world: &mut World) {}

/// Apply the macOS auto-default `frame_rate_limit` at window creation.
/// macOS uses `PresentMode::Immediate` (Mailbox degrades to FIFO on the
/// Metal backend), so the loop tears unless rate-capped. Picks
/// `refresh_rate - 1` from the window's current monitor when the app
/// has not already set a cap. Other platforms leave the cap as `None`.
#[cfg(not(target_arch = "wasm32"))]
fn apply_macos_default_frame_rate_limit(
    world: &mut World,
    window_handle: &std::sync::Arc<winit::window::Window>,
) {
    if !cfg!(target_os = "macos") {
        return;
    }
    if world.resources.render_settings.frame_rate_limit.is_some() {
        return;
    }
    let Some(target) = current_monitor_frame_rate_target(window_handle) else {
        return;
    };
    world.resources.render_settings.frame_rate_limit = Some(target);
    world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline = Some(target);
}

#[cfg(target_arch = "wasm32")]
fn apply_macos_default_frame_rate_limit(
    _world: &mut World,
    _window_handle: &std::sync::Arc<winit::window::Window>,
) {
}

/// Re-apply the macOS auto-default when the window moves to a monitor
/// with a different refresh rate. Only updates when
/// `graphics.frame_rate_limit` still matches the engine-applied
/// baseline; if the app has overridden the cap, the override is left
/// alone.
#[cfg(not(target_arch = "wasm32"))]
fn refresh_macos_frame_rate_limit(
    world: &mut World,
    window_handle: &std::sync::Arc<winit::window::Window>,
) {
    if !cfg!(target_os = "macos") {
        return;
    }
    let baseline = world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline;
    if baseline.is_none() {
        return;
    }
    if world.resources.render_settings.frame_rate_limit != baseline {
        return;
    }
    let Some(target) = current_monitor_frame_rate_target(window_handle) else {
        return;
    };
    world.resources.render_settings.frame_rate_limit = Some(target);
    world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline = Some(target);
}

#[cfg(target_arch = "wasm32")]
fn refresh_macos_frame_rate_limit(
    _world: &mut World,
    _window_handle: &std::sync::Arc<winit::window::Window>,
) {
}

#[cfg(not(target_arch = "wasm32"))]
fn current_monitor_frame_rate_target(
    window_handle: &std::sync::Arc<winit::window::Window>,
) -> Option<f32> {
    let monitor = window_handle.current_monitor()?;
    let millihertz = monitor.refresh_rate_millihertz()?;
    let refresh_hz = millihertz as f32 / 1000.0;
    if refresh_hz > 1.0 {
        Some(refresh_hz - 1.0)
    } else {
        None
    }
}

fn timing_system(world: &mut World, event: &WindowEvent) {
    let _span = tracing::info_span!("timing").entered();
    use web_time::Instant;

    if let WindowEvent::RedrawRequested = event {
        let now = Instant::now();

        let timing = &mut world.resources.window.timing;

        if timing.initial_frame_start_instant.is_none() {
            timing.initial_frame_start_instant = Some(now);
        }

        timing.raw_delta_time = timing
            .last_frame_start_instant
            .map_or(0.0, |last_frame| (now - last_frame).as_secs_f32());

        timing.delta_time = timing.raw_delta_time * timing.time_speed;

        timing.last_frame_start_instant = Some(now);

        if timing.current_frame_start_instant.is_none() {
            timing.current_frame_start_instant = Some(now);
        }

        if let Some(app_start) = timing.initial_frame_start_instant.as_ref() {
            timing.uptime_milliseconds = (now - *app_start).as_millis() as u64;
        }

        timing.frame_counter += 1;
        if let Some(start) = timing.current_frame_start_instant.as_ref() {
            if (now - *start).as_secs_f32() >= 1.0 {
                timing.frames_per_second = timing.frame_counter as f32;
                timing.frame_counter = 0;
                timing.current_frame_start_instant = Some(now);
            }
        } else {
            timing.current_frame_start_instant = Some(now);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_directory_files(
    root: &std::path::Path,
    current: &std::path::Path,
    files: &mut Vec<crate::ecs::input::resources::DroppedFile>,
) {
    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_directory_files(root, &path, files);
        } else if path.is_file() {
            let Ok(data) = std::fs::read(&path) else {
                continue;
            };
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let prefix = root
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| format!("{}/", name))
                .unwrap_or_default();
            let name = format!("{}{}", prefix, relative);
            files.push(crate::ecs::input::resources::DroppedFile { name, data });
        }
    }
}

fn process_dropped_files(world: &mut World) {
    #[cfg(target_arch = "wasm32")]
    {
        WASM_DROPPED_FILES.with(|files| {
            world
                .resources
                .loading
                .dropped_files
                .append(&mut files.borrow_mut());
        });
    }

    let dropped_files: Vec<_> = world.resources.loading.dropped_files.drain(..).collect();
    for file in dropped_files {
        world
            .resources
            .input
            .events
            .push(crate::ecs::input::events::AppEvent::FileDropped(file));
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_capped_dpr(css_width: f64, css_height: f64, raw_dpr: f64) -> f64 {
    let smallest_css = css_width.min(css_height);
    let max_dpr = if smallest_css < 700.0 { 1.5 } else { 2.0 };
    raw_dpr.clamp(1.0, max_dpr)
}

#[cfg(target_arch = "wasm32")]
fn setup_wasm_file_drop() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");
    let canvas = document
        .get_element_by_id("canvas")
        .expect("no canvas element");

    let dragover_closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        event.stop_propagation();
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("dragover", dragover_closure.as_ref().unchecked_ref())
        .expect("failed to add dragover listener");
    dragover_closure.forget();

    let drop_closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        event.stop_propagation();

        let Some(data_transfer) = event.data_transfer() else {
            return;
        };

        let items = data_transfer.items();
        let mut entries: Vec<web_sys::FileSystemEntry> = Vec::new();
        for index in 0..items.length() {
            let Some(item) = items.get(index) else {
                continue;
            };
            if let Ok(Some(entry)) = item.webkit_get_as_entry() {
                entries.push(entry);
            }
        }

        if entries.is_empty() {
            return;
        }

        wasm_bindgen_futures::spawn_local(async move {
            let file_entries = collect_file_entries(entries).await;
            if file_entries.is_empty() {
                return;
            }
            let read_futures = file_entries
                .into_iter()
                .map(|(file_entry, path)| async move { read_file_entry(file_entry, path).await });
            let results = futures::future::join_all(read_futures).await;
            let batch: Vec<crate::ecs::input::resources::DroppedFile> =
                results.into_iter().flatten().collect();
            if !batch.is_empty() {
                WASM_DROPPED_FILES.with(|files| files.borrow_mut().extend(batch));
            }
        });
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("drop", drop_closure.as_ref().unchecked_ref())
        .expect("failed to add drop listener");
    drop_closure.forget();
}

#[cfg(target_arch = "wasm32")]
async fn collect_file_entries(
    entries: Vec<web_sys::FileSystemEntry>,
) -> Vec<(web_sys::FileSystemFileEntry, String)> {
    use wasm_bindgen::JsCast;

    let mut files: Vec<(web_sys::FileSystemFileEntry, String)> = Vec::new();
    let mut worklist: Vec<(web_sys::FileSystemEntry, String)> =
        entries.into_iter().map(|e| (e, String::new())).collect();
    while let Some((entry, prefix)) = worklist.pop() {
        if entry.is_file() {
            let file_entry: web_sys::FileSystemFileEntry = entry.unchecked_into();
            let path = if prefix.is_empty() {
                file_entry.name()
            } else {
                format!("{}/{}", prefix, file_entry.name())
            };
            files.push((file_entry, path));
        } else if entry.is_directory() {
            let directory_entry: web_sys::FileSystemDirectoryEntry = entry.unchecked_into();
            let next_prefix = if prefix.is_empty() {
                directory_entry.name()
            } else {
                format!("{}/{}", prefix, directory_entry.name())
            };
            let reader = directory_entry.create_reader();
            loop {
                let promise = js_sys::Promise::new(&mut |resolve, reject| {
                    let _ = reader.read_entries_with_callback_and_callback(&resolve, &reject);
                });
                let entries_value = match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(value) => value,
                    Err(_) => break,
                };
                let array = js_sys::Array::from(&entries_value);
                if array.length() == 0 {
                    break;
                }
                for index in 0..array.length() {
                    let raw = array.get(index);
                    let child: web_sys::FileSystemEntry = raw.unchecked_into();
                    worklist.push((child, next_prefix.clone()));
                }
            }
        }
    }
    files
}

#[cfg(target_arch = "wasm32")]
async fn read_file_entry(
    file_entry: web_sys::FileSystemFileEntry,
    path: String,
) -> Option<crate::ecs::input::resources::DroppedFile> {
    use wasm_bindgen::JsCast;

    let file_promise = js_sys::Promise::new(&mut |resolve, reject| {
        file_entry.file_with_callback_and_callback(&resolve, &reject);
    });
    let file_value = wasm_bindgen_futures::JsFuture::from(file_promise)
        .await
        .ok()?;
    let file: web_sys::File = file_value.unchecked_into();
    let buffer_value = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .ok()?;
    let array_buffer: js_sys::ArrayBuffer = buffer_value.unchecked_into();
    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    Some(crate::ecs::input::resources::DroppedFile {
        name: path,
        data: bytes,
    })
}

fn forward_window_events(world: &mut World, event: &WindowEvent) {
    use crate::ecs::input::events::AppEvent;

    match event {
        WindowEvent::DroppedFile(path) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if path.is_dir() {
                    let mut files = Vec::new();
                    collect_directory_files(path, path, &mut files);
                    if files.is_empty() {
                        world
                            .resources
                            .input
                            .events
                            .push(AppEvent::FileDroppedPath(path.clone()));
                    } else {
                        for file in files {
                            world
                                .resources
                                .input
                                .events
                                .push(AppEvent::FileDropped(file));
                        }
                    }
                } else {
                    world
                        .resources
                        .input
                        .events
                        .push(AppEvent::FileDroppedPath(path.clone()));
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                world
                    .resources
                    .input
                    .events
                    .push(AppEvent::FileDroppedPath(path.clone()));
            }
        }
        WindowEvent::HoveredFile(path) => {
            world
                .resources
                .input
                .events
                .push(AppEvent::FileHovered(path.clone()));
        }
        WindowEvent::HoveredFileCancelled => {
            world
                .resources
                .input
                .events
                .push(AppEvent::FileHoverCancelled);
        }
        _ => {}
    }

    if world.resources.user_interface.consumed_event {
        return;
    }

    #[cfg(all(target_arch = "wasm32", feature = "audio"))]
    {
        let is_user_gesture = matches!(
            event,
            WindowEvent::KeyboardInput { .. } | WindowEvent::MouseInput { .. }
        );
        if is_user_gesture && !audio_engine_is_initialized(&world.resources.audio) {
            crate::ecs::audio::systems::lazy_initialize_audio_system(world);
        }
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
        world.resources.input.events.push(AppEvent::Keyboard {
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
        world.resources.input.events.push(AppEvent::Mouse {
            button: *button,
            state: *mouse_state,
        });
    }
}

/// Application context for the winit event loop.
pub struct WindowContext {
    /// The ECS world containing all game state.
    pub world: World,
    /// The current application state.
    pub state: Box<dyn State>,
    /// The renderer (None until initialization completes).
    pub renderer: Option<crate::render::wgpu::WgpuRenderer>,
    #[cfg(target_arch = "wasm32")]
    /// Receiver for async renderer creation on web.
    pub renderer_receiver:
        Option<futures::channel::oneshot::Receiver<crate::render::wgpu::WgpuRenderer>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub snapshot_path: Option<std::path::PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    pub snapshot_queued: bool,
    pub initialized: bool,
}

impl winit::application::ApplicationHandler for WindowContext {
    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.world
            .resources
            .input
            .events
            .push(crate::ecs::input::events::AppEvent::Suspended);
        self.renderer = None;
        self.world.resources.window.handle = None;
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.world.resources.window.handle.is_some() {
            return;
        }

        let mut attributes = winit::window::Window::default_attributes();

        #[cfg(not(target_arch = "wasm32"))]
        {
            attributes = attributes
                .with_title(&self.world.resources.window.title)
                .with_resizable(true)
                .with_decorations(true)
                .with_min_inner_size(winit::dpi::LogicalSize::new(400.0, 300.0));

            #[cfg(feature = "assets")]
            if let Some(icon_bytes) = self.world.resources.window.icon_bytes
                && let Ok(icon_image) = image::load_from_memory(icon_bytes)
            {
                let icon_rgba = icon_image.to_rgba8();
                let (width, height) = icon_rgba.dimensions();
                if let Ok(icon) =
                    winit::window::Icon::from_rgba(icon_rgba.into_raw(), width, height)
                {
                    attributes = attributes.with_window_icon(Some(icon));
                }
            }
            self.world.resources.window.applied_title =
                Some(self.world.resources.window.title.clone());
            self.world.resources.window.applied_icon_bytes = self.world.resources.window.icon_bytes;
        }

        #[cfg(target_arch = "wasm32")]
        let (canvas_width, canvas_height) = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let web_window = wgpu::web_sys::window().unwrap();
            let canvas = web_window
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<wgpu::web_sys::HtmlCanvasElement>()
                .unwrap();
            let raw_dpr = web_window.device_pixel_ratio().max(1.0);
            let css_width = web_window
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(800.0);
            let css_height = web_window
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(600.0);
            let dpr = wasm_capped_dpr(css_width, css_height, raw_dpr);
            let physical_width = (css_width * dpr).round() as u32;
            let physical_height = (css_height * dpr).round() as u32;
            canvas.set_width(physical_width);
            canvas.set_height(physical_height);
            attributes = attributes.with_canvas(Some(canvas));
            (physical_width, physical_height)
        };

        #[cfg(target_arch = "wasm32")]
        setup_wasm_file_drop();

        let Ok(window) = event_loop.create_window(attributes) else {
            return;
        };

        let window_handle = std::sync::Arc::new(window);
        self.world.resources.window.cached_scale_factor = window_handle.scale_factor() as f32;
        self.world.resources.window.handle = Some(window_handle.clone());

        crate::ecs::window::resources::refresh_monitors_from_event_loop(
            &mut self.world.resources.monitors,
            event_loop,
        );

        apply_macos_default_frame_rate_limit(&mut self.world, &window_handle);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window_handle.canvas() {
                self.world.resources.window.cached_viewport_size =
                    Some((canvas.width(), canvas.height()));
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let dimension = window_handle.inner_size();

        #[cfg(not(target_arch = "wasm32"))]
        {
            match pollster::block_on(crate::render::core::create_renderer(
                window_handle.clone(),
                dimension.width,
                dimension.height,
            )) {
                Ok(mut renderer) => {
                    if let Err(error) = renderer.configure_with_state(self.state.as_mut()) {
                        tracing::error!("Failed to configure renderer with state: {error}");
                    }
                    self.renderer = Some(renderer);
                }
                Err(error) => {
                    tracing::error!("Failed to create renderer: {error}");
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let (sender, receiver) = futures::channel::oneshot::channel();
            self.renderer_receiver = Some(receiver);

            let window_handle_clone = std::sync::Arc::clone(&window_handle);

            wasm_bindgen_futures::spawn_local(async move {
                match crate::render::core::create_renderer(
                    window_handle_clone,
                    canvas_width,
                    canvas_height,
                )
                .await
                {
                    Ok(renderer) => {
                        if sender.send(renderer).is_err() {
                            tracing::error!("Failed to send renderer");
                        }
                    }
                    Err(error) => {
                        tracing::error!("Failed to create renderer: {error}");
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if !self.initialized {
                self.world.resources.schedules.frame =
                    crate::schedule::build_default_frame_schedule();
                self.world.resources.schedules.retained_ui =
                    crate::schedule::build_default_retained_ui_schedule();
                self.state.initialize(&mut self.world);
                sync_min_window_size(&mut self.world);
                #[cfg(target_arch = "wasm32")]
                crate::ecs::camera::systems::sync_wasm_canvas_size(&mut self.world);
                self.initialized = true;
            }

            self.world
                .resources
                .input
                .events
                .push(crate::ecs::input::events::AppEvent::Resumed);

            if self.world.resources.renderer_state.use_fullscreen
                && let Some(window_handle) = &self.world.resources.window.handle
            {
                window_handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }

            if let Some(window_handle) = &self.world.resources.window.handle {
                window_handle.set_cursor_visible(self.world.resources.renderer_state.show_cursor);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            let mut renderer_received = false;
            if let Some(receiver) = self.renderer_receiver.as_mut()
                && let Ok(Some(mut renderer)) = receiver.try_recv()
            {
                if let Err(error) = renderer.configure_with_state(self.state.as_mut()) {
                    tracing::error!("Failed to configure renderer with state: {error}");
                }
                self.renderer = Some(renderer);
                self.world.resources.schedules.frame =
                    crate::schedule::build_default_frame_schedule();
                self.world.resources.schedules.retained_ui =
                    crate::schedule::build_default_retained_ui_schedule();
                self.state.initialize(&mut self.world);
                sync_min_window_size(&mut self.world);
                #[cfg(target_arch = "wasm32")]
                crate::ecs::camera::systems::sync_wasm_canvas_size(&mut self.world);

                if self.world.resources.renderer_state.use_fullscreen
                    && let Some(window_handle) = &self.world.resources.window.handle
                {
                    window_handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                }

                if let Some(window_handle) = &self.world.resources.window.handle {
                    window_handle
                        .set_cursor_visible(self.world.resources.renderer_state.show_cursor);
                }

                renderer_received = true;
            }
            if renderer_received {
                self.renderer_receiver = None;
            }
        }

        if self.world.resources.window.should_exit
            || matches!(event, winit::event::WindowEvent::CloseRequested)
        {
            event_loop.exit();
            return;
        }

        match event {
            winit::event::WindowEvent::Focused(focused) => {
                self.world.resources.window.is_focused = focused;
                if !focused {
                    self.world.resources.input.mouse.position_initialized = false;
                }
            }
            winit::event::WindowEvent::Moved(_) => {
                if let Some(handle) = self.world.resources.window.handle.clone() {
                    refresh_macos_frame_rate_limit(&mut self.world, &handle);
                }
            }
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(handle) = self.world.resources.window.handle.as_ref() {
                    self.world.resources.window.cached_scale_factor = handle.scale_factor() as f32;
                }
                crate::ecs::window::resources::refresh_monitors_from_event_loop(
                    &mut self.world.resources.monitors,
                    event_loop,
                );
                if let Some(handle) = self.world.resources.window.handle.clone() {
                    refresh_macos_frame_rate_limit(&mut self.world, &handle);
                }
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height })
                if width > 0 && height > 0 =>
            {
                if let Some(handle) = self.world.resources.window.handle.as_ref() {
                    self.world.resources.window.cached_scale_factor = handle.scale_factor() as f32;
                }
                crate::ecs::window::resources::refresh_monitors_from_event_loop(
                    &mut self.world.resources.monitors,
                    event_loop,
                );

                #[cfg(target_arch = "wasm32")]
                let (surface_width, surface_height) = {
                    let web_window = wgpu::web_sys::window().unwrap();
                    let raw_dpr = web_window.device_pixel_ratio().max(1.0);
                    let css_width = web_window
                        .inner_width()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(width as f64);
                    let css_height = web_window
                        .inner_height()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(height as f64);
                    let dpr = wasm_capped_dpr(css_width, css_height, raw_dpr);
                    let physical_width = (css_width * dpr).round() as u32;
                    let physical_height = (css_height * dpr).round() as u32;
                    use winit::platform::web::WindowExtWebSys;
                    if let Some(window) = self.world.resources.window.handle.as_ref()
                        && let Some(canvas) = window.canvas()
                    {
                        canvas.set_width(physical_width);
                        canvas.set_height(physical_height);
                    }
                    (physical_width, physical_height)
                };
                #[cfg(not(target_arch = "wasm32"))]
                let (surface_width, surface_height) = (width, height);

                self.world.resources.window.cached_viewport_size =
                    Some((surface_width, surface_height));

                if let Some(renderer) = &mut self.renderer
                    && let Err(error) = renderer.resize_surface(surface_width, surface_height)
                {
                    tracing::error!("Failed to resize surface: {error}");
                }
            }
            _ => {}
        }

        if let Some(new_state) =
            crate::run::step(&mut self.world, &mut self.state, &mut self.renderer, &event)
        {
            self.state = new_state;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = &self.snapshot_path {
            let uptime = self.world.resources.window.timing.uptime_milliseconds;
            if !self.snapshot_queued && uptime >= 4000 {
                crate::ecs::world::commands::capture_screenshot_to_path(
                    &mut self.world,
                    path.clone(),
                );
                self.snapshot_queued = true;
            } else if self.snapshot_queued && uptime >= 5000 {
                self.world.resources.window.should_exit = true;
            }
        }

        if !self
            .world
            .resources
            .render_settings
            .render_world_to_swapchain
        {
            request_redraw_system(&mut self.world);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let mouse = &mut self.world.resources.input.mouse;
            mouse.raw_mouse_delta.x += delta.0 as f32;
            mouse.raw_mouse_delta.y += delta.1 as f32;
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if self
            .world
            .resources
            .render_settings
            .render_world_to_swapchain
        {
            if let Some(new_state) = step(
                &mut self.world,
                &mut self.state,
                &mut self.renderer,
                &WindowEvent::RedrawRequested,
            ) {
                self.state = new_state;
            }
        } else {
            request_redraw_system(&mut self.world);
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
}

pub(crate) fn request_redraw_system(world: &mut World) {
    let Some(window_handle) = world.resources.window.handle.as_mut() else {
        return;
    };
    let winit::dpi::PhysicalSize { width, height } = window_handle.inner_size();
    if width == 0 || height == 0 {
        return;
    }
    window_handle.request_redraw();
}
