//! Application entry point and main loop.
//!
//! Launch a Nightshade application by implementing [`State`] and calling [`launch`]:
//!
//! ```ignore
//! struct MyGame;
//!
//! impl State for MyGame {
//!     fn title(&self) -> &str { "My Game" }
//!
//!     fn initialize(&mut self, world: &mut World) {
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
//! | `title()` | Window creation | Window title (desktop) |
//! | `icon_bytes()` | Window creation | Window icon PNG (desktop) |
//! | `initialize()` | After renderer ready | Scene setup, spawn entities |
//! | `run_systems()` | Every frame | Game logic, entity updates |
//! | `ui()` | Every frame | egui panels and windows |
//! | `configure_render_graph()` | Once at startup | Custom post-processing |
//! | `update_render_graph()` | Every frame | Dynamic render graph changes |
//! | `handle_event()` | On event bus message | Cross-system communication |
//! | `on_keyboard_input()` | On key press/release | Keyboard handling |
//! | `on_mouse_input()` | On mouse button | Mouse button handling |
//! | `on_gamepad_event()` | On gamepad event | Controller input |
//! | `on_dropped_file()` | On file drop (desktop) | File path handling |
//! | `on_dropped_file_data()` | On file drop (all) | File bytes handling |
//! | `on_suspend()` | App suspended (Android) | Release resources |
//! | `on_resume()` | App resumed (Android) | Restore resources |
//! | `next_state()` | Every frame | State transitions |
//!
//! # Typical Initialize Pattern
//!
//! ```ignore
//! fn initialize(&mut self, world: &mut World) {
//!     // Configure graphics
//!     world.resources.user_interface.enabled = true;
//!     world.resources.graphics.atmosphere = Atmosphere::Sky;
//!     world.resources.graphics.show_grid = true;
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
//! Switch between game states (menu, gameplay, pause) via `next_state()`:
//!
//! ```ignore
//! fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
//!     if self.should_go_to_menu {
//!         Some(Box::new(MenuState::default()))
//!     } else {
//!         None  // Stay in current state
//!     }
//! }
//! ```

#[cfg(feature = "egui")]
mod egui_support;
#[cfg(all(feature = "egui", not(target_arch = "wasm32")))]
pub(crate) use egui_support::initialize_user_interface;
#[cfg(feature = "egui")]
pub(crate) use egui_support::update_egui_scale_factor;
#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
mod mcp_commands;

use crate::ecs::world::World;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

/// Resource IDs for render graph configuration.
///
/// Provided to [`State::configure_render_graph`] for custom pass setup.
pub struct RenderResources {
    /// Main HDR scene color target.
    pub scene_color: crate::render::wgpu::rendergraph::ResourceId,
    /// Depth buffer (reverse-Z format).
    pub depth: crate::render::wgpu::rendergraph::ResourceId,
    /// Compute shader output texture.
    pub compute_output: crate::render::wgpu::rendergraph::ResourceId,
    /// Final swapchain output.
    pub swapchain: crate::render::wgpu::rendergraph::ResourceId,
    /// View-space normals for SSAO.
    pub view_normals: crate::render::wgpu::rendergraph::ResourceId,
    /// Raw SSAO output before blur.
    pub ssao_raw: crate::render::wgpu::rendergraph::ResourceId,
    /// Blurred SSAO output.
    pub ssao: crate::render::wgpu::rendergraph::ResourceId,
    /// Raw SSGI output before blur.
    pub ssgi_raw: crate::render::wgpu::rendergraph::ResourceId,
    /// Blurred SSGI output.
    pub ssgi: crate::render::wgpu::rendergraph::ResourceId,
    /// Raw SSR output before blur.
    pub ssr_raw: crate::render::wgpu::rendergraph::ResourceId,
    /// Blurred SSR output.
    pub ssr: crate::render::wgpu::rendergraph::ResourceId,
    pub surface_width: u32,
    pub surface_height: u32,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_DROPPED_FILES: std::cell::RefCell<Vec<crate::ecs::input::resources::DroppedFile>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Launches the application with the given state.
///
/// Uses OpenXR if the `openxr` feature is enabled, otherwise creates a windowed application.
pub fn launch(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "openxr")]
    {
        crate::xr::launch_xr(state)
    }

    #[cfg(not(feature = "openxr"))]
    {
        launch_windowed(state)
    }
}

#[cfg(all(target_os = "android", feature = "android"))]
pub fn launch_android(
    app: android_activity::AndroidApp,
    state: impl State + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use winit::platform::android::EventLoopBuilderExtAndroid;

    tracing_subscriber::registry()
        .with(tracing_android::layer("nightshade").expect("Failed to create Android log layer"))
        .init();

    let state = Box::new(state);
    let mut world = World::default();
    world.resources.android_app = Some(app.clone());
    crate::filesystem::set_android_app(app.clone());

    let event_loop = EventLoop::builder().with_android_app(app).build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut context = WindowContext {
        state,
        world,
        renderer: None,
        snapshot_path: None,
        snapshot_queued: false,
        initialized: false,
    };
    event_loop.run_app(&mut context)?;

    Ok(())
}

/// Launches a windowed application (desktop or web).
pub fn launch_windowed(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
    let (log_config, log_file_name) = {
        let config = state.log_config();
        let file_name = log_file_name(state.title(), &config);
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

    #[allow(unused_mut)]
    let mut world = World::default();

    #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
    let _mcp_shutdown = {
        let command_queue = crate::mcp::create_mcp_queues();
        world.resources.mcp_command_queue = command_queue.clone();
        crate::mcp::start_mcp_server(command_queue)
    };

    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut context = crate::multi_window::MultiWindowContext::new(state, world);
        event_loop.run_app(&mut context)?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let mut context = WindowContext {
            state,
            world,
            renderer: None,
            renderer_receiver: None,
            initialized: false,
        };
        event_loop.run_app(&mut context)?;
    }

    Ok(())
}

pub(crate) fn step(
    world: &mut World,
    state: &mut Box<dyn State + 'static>,
    renderer: &mut Option<Box<dyn crate::ecs::world::Render>>,
    event: &WindowEvent,
) -> Option<Box<dyn State>> {
    let delta_ms = (world.resources.window.timing.delta_time * 1000.0) as u32;
    let _span = tracing::info_span!("frame", delta_ms).entered();

    handle_event_systems(world, event);

    match event {
        WindowEvent::RedrawRequested => {
            world.core.increment_tick();

            #[cfg(feature = "gamepad")]
            crate::ecs::input::systems::gamepad_input_system(world);
            process_dropped_files(world, state);

            if let Some(renderer) = renderer.as_mut() {
                renderer.initialize_fonts(world);
            }

            #[cfg(feature = "egui")]
            egui_support::create_ui_system(world, state);
            #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
            world.resources.steam.run_callbacks();

            #[cfg(feature = "gamepad")]
            {
                let gamepad_events = world.resources.input.gamepad.events.clone();
                for event in gamepad_events {
                    state.on_gamepad_event(world, event);
                }
            }

            crate::ecs::graphics::systems::day_night_cycle_system(world);
            state.run_systems(world);

            crate::ecs::event_bus::systems::process_events_system(world, |world, message| {
                state.handle_event(world, message);
            });

            #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
            process_mcp_commands_with_state(world, state.as_mut());

            run_frame_systems(world);

            if let Some(new_state) = state.next_state(world) {
                return Some(new_state);
            }

            let window_handle = world.resources.window.handle.as_ref()?;

            if world.resources.retained_ui.enabled {
                let cursor = world
                    .resources
                    .retained_ui
                    .requested_cursor
                    .unwrap_or(winit::window::CursorIcon::Default);
                window_handle.set_cursor(cursor);
            }

            let renderer = renderer.as_mut()?;

            #[cfg(feature = "egui")]
            let ui_frame_output = world.resources.user_interface.frame_output.take();

            let should_render =
                window_handle.inner_size().width > 0 && window_handle.inner_size().height > 0;
            if should_render {
                if let Err(error) = renderer.update_with_state(state.as_mut(), world) {
                    tracing::error!("Failed to update render graph: {error}");
                }

                state.pre_render(&mut **renderer, world);

                #[cfg(feature = "egui")]
                let (ui_output, ui_primitives) = match ui_frame_output {
                    Some((output, primitives)) => (Some(output), Some(primitives)),
                    None => (None, None),
                };
                #[cfg(not(feature = "egui"))]
                let (ui_output, ui_primitives) = (None, None);
                if let Err(error) = renderer.render_frame(world, ui_output, ui_primitives) {
                    tracing::error!("Failed to draw frame: {error}");
                }
            }

            None
        }
        event => {
            forward_window_events(world, state, event);
            None
        }
    }
}

fn handle_event_systems(world: &mut World, event: &WindowEvent) {
    let _span = tracing::info_span!("event_handling").entered();
    #[cfg(feature = "egui")]
    egui_support::ui_event_system(world, event);
    input_event_system(world, event);
    timing_system(world, event);
}

pub(crate) fn run_frame_systems(world: &mut World) {
    let schedule = std::mem::take(&mut world.resources.frame_schedule);
    schedule.run(world);
    world.resources.frame_schedule = schedule;
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
                mouse.position_delta = (current_position - mouse.position) * cursor_scale;
            } else {
                mouse.position_delta = Vec2::new(0.0, 0.0);
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

fn process_dropped_files(world: &mut World, state: &mut Box<dyn State + 'static>) {
    #[cfg(target_arch = "wasm32")]
    {
        WASM_DROPPED_FILES.with(|files| {
            world
                .resources
                .dropped_files
                .append(&mut files.borrow_mut());
        });
    }

    let dropped_files: Vec<_> = world.resources.dropped_files.drain(..).collect();
    if !dropped_files.is_empty() {
        state.on_dropped_files(world, &dropped_files);
    }
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

fn forward_window_events(
    world: &mut World,
    state: &mut Box<dyn State + 'static>,
    event: &WindowEvent,
) {
    match event {
        WindowEvent::DroppedFile(path) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if path.is_dir() {
                    let mut files = Vec::new();
                    collect_directory_files(path, path, &mut files);
                    if files.is_empty() {
                        state.on_dropped_file(world, path);
                    } else {
                        state.on_dropped_files(world, &files);
                    }
                } else {
                    state.on_dropped_file(world, path);
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                state.on_dropped_file(world, path);
            }
        }
        WindowEvent::HoveredFile(path) => {
            state.on_hovered_file(world, path);
        }
        WindowEvent::HoveredFileCancelled => {
            state.on_hovered_file_cancelled(world);
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
        if is_user_gesture && !world.resources.audio.is_initialized() {
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
        state.on_keyboard_input(world, *key_code, *key_state);
    }

    if let WindowEvent::MouseInput {
        button,
        state: mouse_state,
        ..
    } = event
    {
        state.on_mouse_input(world, *mouse_state, *button);
    }
}

/// Controls how log files are rotated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    /// New log file each application launch (timestamp in filename).
    PerSession,
    /// One log file per day.
    Daily,
    /// Single log file, appended to on each launch.
    Never,
}

/// Configuration for file-based logging (desktop only, requires `tracing` feature).
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Directory for log files (relative to working directory).
    pub directory: String,
    /// Log rotation strategy.
    pub rotation: LogRotation,
    /// Default `RUST_LOG` filter when the environment variable is not set.
    pub default_filter: String,
    /// chrono format string for per-session log timestamps.
    /// Default: `"%Y-%m-%d_%H-%M-%S"` produces `2026-03-04_14-30-00`.
    pub timestamp_format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            directory: "logs".to_string(),
            rotation: LogRotation::PerSession,
            default_filter: "info".to_string(),
            timestamp_format: "%Y-%m-%d_%H-%M-%S".to_string(),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
pub fn log_file_name(title: &str, config: &LogConfig) -> String {
    let sanitized: String = title
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    let base_name = if sanitized.is_empty() {
        "app".to_string()
    } else {
        sanitized.to_lowercase()
    };
    match config.rotation {
        LogRotation::PerSession => {
            let timestamp = chrono::Local::now().format(&config.timestamp_format);
            format!("{base_name}_{timestamp}.log")
        }
        LogRotation::Daily | LogRotation::Never => format!("{base_name}.log"),
    }
}

/// Application state trait for game logic and rendering.
///
/// Implement this trait to define your application behavior. The engine calls
/// these methods at appropriate points in the frame loop.
pub trait State {
    /// Window title (desktop only).
    fn title(&self) -> &str {
        "Nightshade"
    }

    /// Logging configuration (desktop only, requires `tracing` feature).
    fn log_config(&self) -> LogConfig {
        LogConfig::default()
    }
    /// Window icon as PNG bytes (desktop only).
    fn icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("../images/icon.png"))
    }
    /// Called once after the renderer is created to set up the initial scene.
    fn initialize(&mut self, _world: &mut World) {}
    /// Returns a new state to transition to, or None to continue with current state.
    fn next_state(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        None
    }
    fn configure_render_graph(
        &mut self,
        _: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        _: &wgpu::Device,
        _: wgpu::TextureFormat,
        _: RenderResources,
    ) {
    }
    /// Build egui UI (requires `egui` feature).
    #[cfg(feature = "egui")]
    fn ui(&mut self, _world: &mut World, _ui_context: &egui::Context) {}
    /// Build egui UI for a secondary application window.
    #[cfg(feature = "egui")]
    fn secondary_ui(
        &mut self,
        _world: &mut World,
        _window_index: usize,
        _ui_context: &egui::Context,
    ) {
    }
    /// Called every frame to run game logic.
    fn run_systems(&mut self, _world: &mut World) {}
    /// Called before main rendering to allow custom world rendering.
    fn pre_render(&mut self, _renderer: &mut dyn crate::ecs::world::Render, _world: &mut World) {}
    /// Update render graph state each frame.
    fn update_render_graph(
        &mut self,
        _graph: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        _world: &World,
    ) {
    }
    /// Handle events from the event bus.
    fn handle_event(&mut self, _world: &mut World, _message: &crate::ecs::world::events::Message) {}
    /// Handle keyboard input (not consumed by UI).
    fn on_keyboard_input(
        &mut self,
        _world: &mut World,
        _key_code: winit::keyboard::KeyCode,
        _key_state: winit::event::ElementState,
    ) {
    }
    /// Handle file dropped onto window (desktop only, provides path).
    fn on_dropped_file(&mut self, _world: &mut World, _path: &std::path::Path) {}
    /// Handle file dropped onto window (web-compatible, provides data).
    fn on_dropped_file_data(&mut self, _world: &mut World, _name: &str, _data: &[u8]) {}
    /// Handle a batch of files dropped together. Default delegates to
    /// `on_dropped_file_data` per file. Override to handle multi-file formats
    /// like `.gltf` plus external `.bin` and texture files.
    fn on_dropped_files(
        &mut self,
        world: &mut World,
        files: &[crate::ecs::input::resources::DroppedFile],
    ) {
        for file in files {
            self.on_dropped_file_data(world, &file.name, &file.data);
        }
    }
    /// Handle file hover over window.
    fn on_hovered_file(&mut self, _world: &mut World, _path: &std::path::Path) {}
    /// Handle file hover cancelled.
    fn on_hovered_file_cancelled(&mut self, _world: &mut World) {}
    /// Handle gamepad events (requires `gamepad` feature).
    #[cfg(feature = "gamepad")]
    fn on_gamepad_event(&mut self, _world: &mut World, _event: gilrs::Event) {}
    /// Called when the application is suspended (Android lifecycle).
    fn on_suspend(&mut self, _world: &mut World) {}
    /// Called when the application is resumed (Android lifecycle).
    fn on_resume(&mut self, _world: &mut World) {}
    /// Handle mouse button input (not consumed by UI).
    fn on_mouse_input(
        &mut self,
        _world: &mut World,
        _state: winit::event::ElementState,
        _button: winit::event::MouseButton,
    ) {
    }
    /// Intercept an MCP command before the engine processes it.
    ///
    /// Return `Some(response)` to handle the command yourself (engine skips it),
    /// or `None` to let the engine handle it with the default behavior.
    #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
    fn handle_mcp_command(
        &mut self,
        _world: &mut World,
        _command: &crate::mcp::McpCommand,
    ) -> Option<crate::mcp::McpResponse> {
        None
    }
    /// Called after an MCP command has been processed (either by the engine or by `handle_mcp_command`).
    #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
    fn after_mcp_command(
        &mut self,
        _world: &mut World,
        _command: &crate::mcp::McpCommand,
        _response: &crate::mcp::McpResponse,
    ) {
    }
}

/// Application context for the winit event loop.
pub struct WindowContext {
    /// The ECS world containing all game state.
    pub world: World,
    /// The current application state.
    pub state: Box<dyn State>,
    /// The renderer (None until initialization completes).
    pub renderer: Option<Box<dyn crate::ecs::world::Render>>,
    #[cfg(target_arch = "wasm32")]
    /// Receiver for async renderer creation on web.
    pub renderer_receiver:
        Option<futures::channel::oneshot::Receiver<Box<dyn crate::ecs::world::Render>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub snapshot_path: Option<std::path::PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    pub snapshot_queued: bool,
    pub initialized: bool,
}

impl winit::application::ApplicationHandler for WindowContext {
    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.state.on_suspend(&mut self.world);
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
            attributes = attributes.with_title(self.state.title());

            #[cfg(feature = "assets")]
            if let Some(icon_bytes) = self.state.icon_bytes()
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
        }

        #[cfg(target_arch = "wasm32")]
        let (canvas_width, canvas_height) = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = wgpu::web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<wgpu::web_sys::HtmlCanvasElement>()
                .unwrap();
            let canvas_width = canvas.width();
            let canvas_height = canvas.height();
            attributes = attributes.with_canvas(Some(canvas));
            (canvas_width, canvas_height)
        };

        #[cfg(target_arch = "wasm32")]
        setup_wasm_file_drop();

        let Ok(window) = event_loop.create_window(attributes) else {
            return;
        };

        let window_handle = std::sync::Arc::new(window);
        self.world.resources.window.cached_scale_factor = window_handle.scale_factor() as f32;
        self.world.resources.window.handle = Some(window_handle.clone());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window_handle.canvas() {
                let client_width = canvas.client_width() as u32;
                let client_height = canvas.client_height() as u32;
                self.world.resources.window.cached_viewport_size =
                    Some((client_width, client_height));
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let dimension = window_handle.inner_size();

        #[cfg(all(not(target_arch = "wasm32"), feature = "async_runtime"))]
        {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            match runtime.block_on(crate::render::core::create_renderer(
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

        #[cfg(all(not(target_arch = "wasm32"), not(feature = "async_runtime")))]
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
            #[cfg(feature = "egui")]
            egui_support::initialize_user_interface(&mut self.world);

            if !self.initialized {
                self.world.resources.frame_schedule =
                    crate::schedule::build_default_frame_schedule();
                self.state.initialize(&mut self.world);
                crate::ecs::camera::systems::update_camera_aspect_ratios(&mut self.world);
                self.initialized = true;
            }

            self.state.on_resume(&mut self.world);

            if self.world.resources.graphics.use_fullscreen
                && let Some(window_handle) = &self.world.resources.window.handle
            {
                window_handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }

            if let Some(window_handle) = &self.world.resources.window.handle {
                window_handle.set_cursor_visible(self.world.resources.graphics.show_cursor);
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
                #[cfg(feature = "egui")]
                egui_support::initialize_user_interface(&mut self.world);
                self.world.resources.frame_schedule =
                    crate::schedule::build_default_frame_schedule();
                self.state.initialize(&mut self.world);
                crate::ecs::camera::systems::update_camera_aspect_ratios(&mut self.world);

                if self.world.resources.graphics.use_fullscreen
                    && let Some(window_handle) = &self.world.resources.window.handle
                {
                    window_handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                }

                if let Some(window_handle) = &self.world.resources.window.handle {
                    window_handle.set_cursor_visible(self.world.resources.graphics.show_cursor);
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
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(handle) = self.world.resources.window.handle.as_ref() {
                    self.world.resources.window.cached_scale_factor = handle.scale_factor() as f32;
                }
                #[cfg(feature = "egui")]
                egui_support::update_egui_scale_factor(&mut self.world);
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height })
                if width > 0 && height > 0 =>
            {
                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowExtWebSys;
                    if let Some(window) = self.world.resources.window.handle.as_ref()
                        && let Some(canvas) = window.canvas()
                    {
                        canvas.set_width(width);
                        canvas.set_height(height);
                    }
                }

                self.world.resources.window.cached_viewport_size = Some((width, height));
                if let Some(handle) = self.world.resources.window.handle.as_ref() {
                    self.world.resources.window.cached_scale_factor = handle.scale_factor() as f32;
                }

                if let Some(renderer) = &mut self.renderer
                    && let Err(error) = renderer.resize_surface(width, height)
                {
                    tracing::error!("Failed to resize surface: {error}");
                }

                #[cfg(feature = "egui")]
                egui_support::update_egui_scale_factor(&mut self.world);
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

        request_redraw_system(&mut self.world);
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
        request_redraw_system(&mut self.world);
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

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
pub(crate) fn process_mcp_commands_with_state(world: &mut World, state: &mut dyn State) {
    let commands: Vec<crate::mcp::PendingCommand> = {
        let mut queue = world.resources.mcp_command_queue.lock().unwrap();
        queue.drain(..).collect()
    };

    for (command, response_sender) in commands {
        let response = match state.handle_mcp_command(world, &command) {
            Some(response) => response,
            None => mcp_commands::process_single_mcp_command(world, command.clone()),
        };

        state.after_mcp_command(world, &command, &response);
        let _ = response_sender.send(response);
    }
}
