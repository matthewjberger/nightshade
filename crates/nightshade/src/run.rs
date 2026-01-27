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
//! | `immediate_ui()` | Every frame | Custom UI without egui |
//! | `configure_render_graph()` | Once at startup | Custom post-processing |
//! | `update_render_graph()` | Every frame | Dynamic render graph changes |
//! | `handle_event()` | On event bus message | Cross-system communication |
//! | `on_keyboard_input()` | On key press/release | Keyboard handling |
//! | `on_mouse_input()` | On mouse button | Mouse button handling |
//! | `on_gamepad_event()` | On gamepad event | Controller input |
//! | `on_dropped_file()` | On file drop (desktop) | File path handling |
//! | `on_dropped_file_data()` | On file drop (all) | File bytes handling |
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

/// Launches a windowed application (desktop or web).
pub fn launch_windowed(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
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

        let file_appender = tracing_appender::rolling::never("logs", "nightshade.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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

        let file_appender = tracing_appender::rolling::never("logs", "nightshade.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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

        let file_appender = tracing_appender::rolling::never("logs", "nightshade.log");
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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

        let file_appender = tracing_appender::rolling::never("logs", "nightshade.log");
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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
    {
        let (command_queue, response_queue) = crate::mcp::create_mcp_queues();
        world.resources.mcp_command_queue = command_queue.clone();
        world.resources.mcp_response_queue = response_queue.clone();
        crate::mcp::start_mcp_server(command_queue, response_queue);
    }

    let mut context = WindowContext {
        state,
        world,
        renderer: None,
        #[cfg(target_arch = "wasm32")]
        renderer_receiver: None,
    };

    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut context)?;

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
            #[cfg(feature = "gamepad")]
            crate::ecs::input::systems::gamepad_input_system(world);
            process_dropped_files(world, state);

            if let Some(renderer) = renderer.as_mut() {
                renderer.initialize_fonts(world);
            }

            #[cfg(feature = "egui")]
            create_ui_system(world, state);
            create_immediate_ui_system(world, state);

            #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
            world.resources.steam.run_callbacks();

            #[cfg(feature = "gamepad")]
            {
                let gamepad_events = world.resources.input.gamepad.events.clone();
                for event in gamepad_events {
                    state.on_gamepad_event(world, event);
                }
            }

            state.run_systems(world);

            crate::ecs::event_bus::systems::process_events_system(world, |world, message| {
                state.handle_event(world, message);
            });

            run_frame_systems(world);

            if let Some(new_state) = state.next_state(world) {
                return Some(new_state);
            }

            let window_handle = world.resources.window.handle.as_ref()?;
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
    ui_event_system(world, event);
    input_event_system(world, event);
    timing_system(world, event);
}

pub(crate) fn run_frame_systems(world: &mut World) {
    let _span = tracing::info_span!("systems").entered();

    #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
    process_mcp_commands(world);

    #[cfg(feature = "audio")]
    {
        crate::ecs::audio::systems::initialize_audio_system(world);
        crate::ecs::audio::systems::update_audio_system(world);
    }
    crate::ecs::camera::systems::update_camera_aspect_ratios(world);
    #[cfg(feature = "physics")]
    crate::ecs::physics::systems::run_physics_systems(world);

    #[cfg(feature = "scripting")]
    {
        let mut runtime = std::mem::take(&mut world.resources.script_runtime);
        crate::ecs::script::systems::run_scripts_system(world, &mut runtime);
        world.resources.script_runtime = runtime;
    }

    let delta_time = world.resources.window.timing.delta_time;
    crate::ecs::animation::systems::update_animation_players(world, delta_time);
    crate::ecs::animation::systems::apply_animations(world);

    crate::ecs::transform::systems::run_systems(world);
    crate::ecs::mesh::systems::update_instanced_mesh_caches_system(world);
    crate::ecs::text::systems::sync_hud_text_system(world);
    crate::ecs::text::systems::sync_immediate_ui_text_system(world);
    crate::ecs::input::systems::reset_mouse_system(world);
    crate::ecs::input::systems::reset_touch_system(world);
    crate::ecs::world::process_commands_system(world);
    crate::ecs::world::cleanup_unused_resources_system(world);
}

#[cfg(feature = "egui")]
fn ui_event_system(world: &mut World, window_event: &WindowEvent) {
    let _span = tracing::info_span!("ui_event").entered();
    let Some(gui_state) = &mut world.resources.user_interface.state else {
        return;
    };
    let Some(window_handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    world.resources.user_interface.consumed_event = gui_state
        .on_window_event(window_handle, window_event)
        .consumed;
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
            let last_position = mouse.position;
            let current_position = Vec2::new(position.x as _, position.y as _);
            mouse.position = current_position;
            mouse.position_delta = current_position - last_position;
            mouse.state.set(MouseState::MOVED, true);
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
    for file in dropped_files {
        state.on_dropped_file_data(world, &file.name, &file.data);
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
        let Some(files) = data_transfer.files() else {
            return;
        };

        for index in 0..files.length() {
            let Some(file) = files.get(index) else {
                continue;
            };

            let file_name = file.name();
            let file_reader = web_sys::FileReader::new().expect("failed to create FileReader");

            let onload_closure = {
                let file_name = file_name.clone();
                let file_reader = file_reader.clone();

                Closure::once(Box::new(move |_event: web_sys::ProgressEvent| {
                    let result = file_reader.result().expect("failed to get result");
                    let array_buffer = result
                        .dyn_into::<js_sys::ArrayBuffer>()
                        .expect("result is not ArrayBuffer");
                    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                    let data = uint8_array.to_vec();

                    WASM_DROPPED_FILES.with(|files| {
                        files
                            .borrow_mut()
                            .push(crate::ecs::input::resources::DroppedFile {
                                name: file_name,
                                data,
                            });
                    });
                }) as Box<dyn FnOnce(_)>)
            };

            file_reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
            onload_closure.forget();

            file_reader
                .read_as_array_buffer(&file)
                .expect("failed to read file");
        }
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("drop", drop_closure.as_ref().unchecked_ref())
        .expect("failed to add drop listener");
    drop_closure.forget();
}

fn forward_window_events(
    world: &mut World,
    state: &mut Box<dyn State + 'static>,
    event: &WindowEvent,
) {
    match event {
        WindowEvent::DroppedFile(path) => {
            state.on_dropped_file(world, path);
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

/// Application state trait for game logic and rendering.
///
/// Implement this trait to define your application behavior. The engine calls
/// these methods at appropriate points in the frame loop.
pub trait State {
    /// Window title (desktop only).
    fn title(&self) -> &str {
        "Nightshade"
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
    /// Configure custom render passes. Called once during initialization.
    fn configure_render_graph(
        &mut self,
        graph: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let compute_blit_pass = crate::render::wgpu::passes::BlitPass::new(device, surface_format)
            .with_name("default_compute_blit");
        let _ = graph.add_pass(
            Box::new(compute_blit_pass),
            &[
                ("input", resources.scene_color),
                ("output", resources.compute_output),
            ],
        );

        let swapchain_blit_pass =
            crate::render::wgpu::passes::BlitPass::new(device, surface_format)
                .with_name("default_swapchain_blit");
        let _ = graph.add_pass(
            Box::new(swapchain_blit_pass),
            &[
                ("input", resources.compute_output),
                ("output", resources.swapchain),
            ],
        );
    }
    /// Build egui UI (requires `egui` feature).
    #[cfg(feature = "egui")]
    fn ui(&mut self, _world: &mut World, _ui_context: &egui::Context) {}
    /// Build immediate-mode UI without egui.
    fn immediate_ui(&mut self, _world: &mut World, _ui: &mut crate::ecs::ui::ImmediateUi) {}
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
    /// Handle file hover over window.
    fn on_hovered_file(&mut self, _world: &mut World, _path: &std::path::Path) {}
    /// Handle file hover cancelled.
    fn on_hovered_file_cancelled(&mut self, _world: &mut World) {}
    /// Handle gamepad events (requires `gamepad` feature).
    #[cfg(feature = "gamepad")]
    fn on_gamepad_event(&mut self, _world: &mut World, _event: gilrs::Event) {}
    /// Handle mouse button input (not consumed by UI).
    fn on_mouse_input(
        &mut self,
        _world: &mut World,
        _state: winit::event::ElementState,
        _button: winit::event::MouseButton,
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
}

impl winit::application::ApplicationHandler for WindowContext {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
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

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.world.resources.window.cached_viewport_size =
                Some((dimension.width, dimension.height));
        }

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
            initialize_user_interface(&mut self.world);
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
                initialize_user_interface(&mut self.world);
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
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                #[cfg(feature = "egui")]
                if let Some(gui_state) = self.world.resources.user_interface.state.as_mut() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let scale_factor =
                            self.world.resources.graphics.ui_scale.unwrap_or_else(|| {
                                self.world
                                    .resources
                                    .window
                                    .handle
                                    .as_ref()
                                    .map(|w| w.scale_factor() as f32)
                                    .unwrap_or(1.0)
                            });
                        gui_state.egui_ctx().set_pixels_per_point(scale_factor);
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        gui_state.egui_ctx().set_pixels_per_point(1.0);
                    }
                }
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

                if let Some(renderer) = &mut self.renderer
                    && let Err(error) = renderer.resize_surface(width, height)
                {
                    tracing::error!("Failed to resize surface: {error}");
                }

                #[cfg(feature = "egui")]
                if let Some(gui_state) = self.world.resources.user_interface.state.as_mut() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let scale_factor =
                            self.world.resources.graphics.ui_scale.unwrap_or_else(|| {
                                self.world
                                    .resources
                                    .window
                                    .handle
                                    .as_ref()
                                    .map(|w| w.scale_factor() as f32)
                                    .unwrap_or(1.0)
                            });
                        gui_state.egui_ctx().set_pixels_per_point(scale_factor);
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        gui_state.egui_ctx().set_pixels_per_point(1.0);
                    }
                }
            }
            _ => {}
        }

        if let Some(new_state) =
            crate::run::step(&mut self.world, &mut self.state, &mut self.renderer, &event)
        {
            self.state = new_state;
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

fn request_redraw_system(world: &mut World) {
    let Some(window_handle) = world.resources.window.handle.as_mut() else {
        return;
    };
    let winit::dpi::PhysicalSize { width, height } = window_handle.inner_size();
    if width == 0 || height == 0 {
        return;
    }
    window_handle.request_redraw();
}

#[cfg(feature = "egui")]
fn initialize_user_interface(world: &mut World) {
    let window_handle = {
        let Some(window_handle) = world.resources.window.handle.as_mut() else {
            return;
        };
        window_handle.clone()
    };

    let gui_context = egui::Context::default();

    #[cfg(target_arch = "wasm32")]
    {
        gui_context.set_pixels_per_point(1.0);
    }

    let viewport_id = gui_context.viewport_id();
    let gui_state = egui_winit::State::new(
        gui_context,
        viewport_id,
        &window_handle,
        Some(window_handle.scale_factor() as _),
        Some(winit::window::Theme::Dark),
        None,
    );

    egui_extras::install_image_loaders(gui_state.egui_ctx());

    world.resources.user_interface.state = Some(gui_state);
}

#[cfg(feature = "egui")]
fn create_ui_system(world: &mut World, state: &mut Box<dyn State + 'static>) {
    let _span = tracing::info_span!("ui").entered();
    if !world.resources.user_interface.enabled {
        return;
    }

    let ui = {
        let Some(gui_state) = world.resources.user_interface.state.as_mut() else {
            return;
        };
        let Some(window_handle) = world.resources.window.handle.as_ref() else {
            return;
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let scale_factor = world
                .resources
                .graphics
                .ui_scale
                .unwrap_or_else(|| window_handle.scale_factor() as f32);
            gui_state.egui_ctx().set_pixels_per_point(scale_factor);
        }

        #[cfg(not(target_arch = "wasm32"))]
        let gui_input = gui_state.take_egui_input(window_handle);

        #[cfg(target_arch = "wasm32")]
        let gui_input = {
            use winit::platform::web::WindowExtWebSys;
            let mut input = gui_state.take_egui_input(window_handle);
            if let Some(canvas) = window_handle.canvas() {
                let dpr = window_handle.scale_factor() as f32;
                let canvas_size =
                    egui::vec2(canvas.width() as f32 / dpr, canvas.height() as f32 / dpr);
                input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, canvas_size));
            }
            input
        };

        gui_state.egui_ctx().begin_pass(gui_input);
        gui_state.egui_ctx().clone()
    };

    state.ui(world, &ui);

    let Some(gui_state) = world.resources.user_interface.state.as_mut() else {
        return;
    };
    let Some(window_handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    let output = ui.end_pass();
    gui_state.handle_platform_output(window_handle, output.platform_output.clone());
    let paint_jobs = ui.tessellate(output.shapes.clone(), output.pixels_per_point);
    world.resources.user_interface.frame_output = Some((output, paint_jobs));
}

fn create_immediate_ui_system(world: &mut World, state: &mut Box<dyn State + 'static>) {
    let _span = tracing::info_span!("immediate_ui").entered();

    if !world.resources.immediate_ui.enabled {
        return;
    }

    let Some(window_handle) = world.resources.window.handle.as_ref() else {
        return;
    };

    let size = window_handle.inner_size();
    let screen_size = crate::ecs::world::Vec2::new(size.width as f32, size.height as f32);
    let mouse_pos = crate::ecs::world::Vec2::new(
        world.resources.input.mouse.position.x,
        world.resources.input.mouse.position.y,
    );
    let mouse_down = world
        .resources
        .input
        .mouse
        .state
        .contains(crate::ecs::world::resources::MouseState::LEFT_CLICKED);
    let mouse_just_pressed = world
        .resources
        .input
        .mouse
        .state
        .contains(crate::ecs::world::resources::MouseState::LEFT_JUST_PRESSED);
    let mouse_just_released = world
        .resources
        .input
        .mouse
        .state
        .contains(crate::ecs::world::resources::MouseState::LEFT_JUST_RELEASED);

    let scroll_delta = world.resources.input.mouse.wheel_delta;
    world.resources.immediate_ui.begin_frame(
        screen_size,
        mouse_pos,
        mouse_down,
        mouse_just_pressed,
        mouse_just_released,
        scroll_delta,
    );

    for character in &world.resources.input.keyboard.frame_chars {
        world.resources.immediate_ui.add_char_input(*character);
    }
    for (key_code, pressed) in &world.resources.input.keyboard.frame_keys {
        world
            .resources
            .immediate_ui
            .add_key_input(*key_code, *pressed);
    }

    let mut ui = std::mem::take(&mut world.resources.immediate_ui);
    state.immediate_ui(world, &mut ui);
    world.resources.immediate_ui = ui;

    world.resources.immediate_ui.end_frame();
    world.resources.input.keyboard.clear_frame_input();
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn find_mesh_child(world: &World, pivot: freecs::Entity) -> Option<freecs::Entity> {
    use crate::ecs::world::{MATERIAL_REF, PARENT};

    if world.entity_has_components(pivot, MATERIAL_REF) {
        return Some(pivot);
    }

    world.query_entities(PARENT | MATERIAL_REF).find(|&child| {
        world
            .get_parent(child)
            .map(|p| p.0 == Some(pivot))
            .unwrap_or(false)
    })
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn spawn_mesh_by_type(world: &mut World, mesh_type: &str) -> Option<freecs::Entity> {
    match mesh_type.to_lowercase().as_str() {
        "cube" => Some(crate::ecs::world::spawn_cube_at(
            world,
            nalgebra_glm::Vec3::zeros(),
        )),
        "sphere" => Some(crate::ecs::world::spawn_sphere_at(
            world,
            nalgebra_glm::Vec3::zeros(),
        )),
        "cylinder" => Some(crate::ecs::world::spawn_cylinder_at(
            world,
            nalgebra_glm::Vec3::zeros(),
        )),
        "cone" => Some(crate::ecs::world::spawn_cone_at(
            world,
            nalgebra_glm::Vec3::zeros(),
        )),
        "plane" => Some(crate::ecs::world::spawn_plane_at(
            world,
            nalgebra_glm::Vec3::zeros(),
        )),
        "torus" => Some(crate::ecs::world::spawn_torus_at(
            world,
            nalgebra_glm::Vec3::zeros(),
        )),
        _ => None,
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
struct SpawnEntityParams {
    name: String,
    mesh: String,
    position: [f32; 3],
    scale: [f32; 3],
    color: Option<[f32; 4]>,
    emissive: Option<[f32; 3]>,
    parent_name: Option<String>,
    alpha_mode: Option<String>,
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_spawn_entity(world: &mut World, params: SpawnEntityParams) -> crate::mcp::McpResponse {
    let SpawnEntityParams {
        name,
        mesh,
        position,
        scale,
        color,
        emissive,
        parent_name,
        alpha_mode,
    } = params;
    use crate::ecs::transform::components::Parent;
    use crate::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, NAME, PARENT};
    use crate::mcp::McpResponse;

    if world.resources.entity_names.contains_key(&name) {
        return McpResponse::Error(format!("Entity '{}' already exists", name));
    }

    let parent_entity = if let Some(ref pname) = parent_name {
        let Some(&parent) = world.resources.entity_names.get(pname) else {
            return McpResponse::Error(format!("Parent entity '{}' not found", pname));
        };
        Some(parent)
    } else {
        None
    };

    let pivot = world.spawn_entities(LOCAL_TRANSFORM | GLOBAL_TRANSFORM | PARENT, 1)[0];
    if let Some(transform) = world.get_local_transform_mut(pivot) {
        transform.translation = nalgebra_glm::Vec3::new(position[0], position[1], position[2]);
    }
    crate::ecs::transform::commands::mark_local_transform_dirty(world, pivot);

    if let Some(parent) = parent_entity {
        world.update_parent(pivot, Some(Parent(Some(parent))));
    }

    let Some(entity) = spawn_mesh_by_type(world, &mesh) else {
        crate::ecs::world::commands::despawn_recursive_immediate(world, pivot);
        return McpResponse::Error(format!(
            "Unknown mesh type '{}'. Valid types: Cube, Sphere, Cylinder, Cone, Plane, Torus",
            mesh
        ));
    };

    world.update_parent(entity, Some(Parent(Some(pivot))));

    if scale != [1.0, 1.0, 1.0] {
        if let Some(transform) = world.get_local_transform_mut(pivot) {
            transform.scale = nalgebra_glm::Vec3::new(scale[0], scale[1], scale[2]);
        }
        crate::ecs::transform::commands::mark_local_transform_dirty(world, pivot);
    }

    let material_name = name.clone();
    let mut material = crate::ecs::material::components::Material::default();
    if let Some(c) = color {
        material.base_color = c;
    }
    if let Some(e) = emissive {
        material.emissive_factor = e;
    }
    if let Some(ref alpha) = alpha_mode {
        material.alpha_mode = match alpha.to_lowercase().as_str() {
            "opaque" => crate::ecs::material::components::AlphaMode::Opaque,
            "mask" => crate::ecs::material::components::AlphaMode::Mask,
            "blend" => crate::ecs::material::components::AlphaMode::Blend,
            _ => crate::ecs::material::components::AlphaMode::Opaque,
        };
    }
    crate::ecs::material::resources::material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(
        entity,
        crate::ecs::material::components::MaterialRef::new(material_name),
    );

    world.add_components(pivot, NAME);
    world.set_name(pivot, crate::ecs::name::components::Name(name.clone()));
    world.resources.entity_names.insert(name.clone(), pivot);

    let parent_info = parent_name
        .map(|p| format!(" (child of '{}')", p))
        .unwrap_or_default();
    McpResponse::Success(format!(
        "Spawned entity '{}' with {} mesh at {:?}{}",
        name, mesh, position, parent_info
    ))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_despawn_entity(world: &mut World, name: String) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    if let Some(entity) = world.resources.entity_names.remove(&name) {
        crate::ecs::world::commands::despawn_recursive_immediate(world, entity);
        McpResponse::Success(format!("Despawned entity '{}'", name))
    } else {
        McpResponse::Error(format!("Entity '{}' not found", name))
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_position(
    world: &mut World,
    name: String,
    position: [f32; 3],
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(transform) = world.get_local_transform_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no transform", name));
    };

    transform.translation = nalgebra_glm::Vec3::new(position[0], position[1], position[2]);
    crate::ecs::transform::systems::mark_local_transform_dirty(world, entity);
    McpResponse::Success(format!("Set position of '{}' to {:?}", name, position))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_rotation(
    world: &mut World,
    name: String,
    rotation: [f32; 3],
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(transform) = world.get_local_transform_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no transform", name));
    };

    let qx = nalgebra_glm::quat_angle_axis(rotation[0], &nalgebra_glm::Vec3::x());
    let qy = nalgebra_glm::quat_angle_axis(rotation[1], &nalgebra_glm::Vec3::y());
    let qz = nalgebra_glm::quat_angle_axis(rotation[2], &nalgebra_glm::Vec3::z());
    transform.rotation = nalgebra_glm::quat_normalize(&(qz * qy * qx));
    crate::ecs::transform::systems::mark_local_transform_dirty(world, entity);
    McpResponse::Success(format!("Set rotation of '{}' to {:?}", name, rotation))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_scale(world: &mut World, name: String, scale: [f32; 3]) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(transform) = world.get_local_transform_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no transform", name));
    };

    transform.scale = nalgebra_glm::Vec3::new(scale[0], scale[1], scale[2]);
    crate::ecs::transform::systems::mark_local_transform_dirty(world, entity);
    McpResponse::Success(format!("Set scale of '{}' to {:?}", name, scale))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_material_color(
    world: &mut World,
    name: String,
    color: [f32; 4],
) -> crate::mcp::McpResponse {
    use crate::ecs::generational_registry::{registry_entry_by_name, registry_entry_by_name_mut};
    use crate::mcp::McpResponse;

    let Some(&pivot) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(mesh) = find_mesh_child(world, pivot) else {
        return McpResponse::Error(format!("Entity '{}' has no mesh with material", name));
    };

    let Some(material_ref) = world.get_material_ref(mesh) else {
        return McpResponse::Error(format!("Entity '{}' has no material reference", name));
    };

    let current_material_name = material_ref.name.clone();
    let ref_count = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&current_material_name)
        .map(|&index| {
            world
                .resources
                .material_registry
                .registry
                .reference_count(index)
        })
        .unwrap_or(0);

    let target_material_name = if ref_count > 1 {
        let unique_name = name.clone();
        if !world
            .resources
            .material_registry
            .registry
            .name_to_index
            .contains_key(&unique_name)
        {
            let new_material = registry_entry_by_name(
                &world.resources.material_registry.registry,
                &current_material_name,
            )
            .cloned()
            .unwrap_or_default();
            crate::ecs::material::resources::material_registry_insert(
                &mut world.resources.material_registry,
                unique_name.clone(),
                new_material,
            );
        }
        if let Some(&old_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&current_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .remove_reference(old_index);
        }
        if let Some(&new_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&unique_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(new_index);
        }
        world.set_material_ref(
            mesh,
            crate::ecs::material::components::MaterialRef::new(unique_name.clone()),
        );
        unique_name
    } else {
        current_material_name
    };

    let Some(material) = registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        &target_material_name,
    ) else {
        return McpResponse::Error(format!(
            "Material '{}' not found for entity '{}'",
            target_material_name, name
        ));
    };

    material.base_color = [color[0], color[1], color[2], color[3]];
    McpResponse::Success(format!("Set material color of '{}' to {:?}", name, color))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_emissive(
    world: &mut World,
    name: String,
    emissive: [f32; 3],
) -> crate::mcp::McpResponse {
    use crate::ecs::generational_registry::{registry_entry_by_name, registry_entry_by_name_mut};
    use crate::mcp::McpResponse;

    let Some(&pivot) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(mesh) = find_mesh_child(world, pivot) else {
        return McpResponse::Error(format!("Entity '{}' has no mesh with material", name));
    };

    let Some(material_ref) = world.get_material_ref(mesh) else {
        return McpResponse::Error(format!("Entity '{}' has no material reference", name));
    };

    let current_material_name = material_ref.name.clone();
    let ref_count = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&current_material_name)
        .map(|&index| {
            world
                .resources
                .material_registry
                .registry
                .reference_count(index)
        })
        .unwrap_or(0);

    let target_material_name = if ref_count > 1 {
        let unique_name = name.clone();
        if !world
            .resources
            .material_registry
            .registry
            .name_to_index
            .contains_key(&unique_name)
        {
            let new_material = registry_entry_by_name(
                &world.resources.material_registry.registry,
                &current_material_name,
            )
            .cloned()
            .unwrap_or_default();
            crate::ecs::material::resources::material_registry_insert(
                &mut world.resources.material_registry,
                unique_name.clone(),
                new_material,
            );
        }
        if let Some(&old_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&current_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .remove_reference(old_index);
        }
        if let Some(&new_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&unique_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(new_index);
        }
        world.set_material_ref(
            mesh,
            crate::ecs::material::components::MaterialRef::new(unique_name.clone()),
        );
        unique_name
    } else {
        current_material_name
    };

    let Some(material) = registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        &target_material_name,
    ) else {
        return McpResponse::Error(format!(
            "Material '{}' not found for entity '{}'",
            target_material_name, name
        ));
    };

    material.emissive_factor = emissive;
    McpResponse::Success(format!("Set emissive of '{}' to {:?}", name, emissive))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_parent(
    world: &mut World,
    name: String,
    parent_name: Option<String>,
) -> crate::mcp::McpResponse {
    use crate::ecs::transform::components::Parent;
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let parent_entity = if let Some(ref pname) = parent_name {
        if pname.is_empty() {
            None
        } else {
            let Some(&parent) = world.resources.entity_names.get(pname) else {
                return McpResponse::Error(format!("Parent entity '{}' not found", pname));
            };
            Some(parent)
        }
    } else {
        None
    };

    world.set_parent(entity, Parent(parent_entity));
    crate::ecs::transform::commands::mark_local_transform_dirty(world, entity);

    match parent_name {
        Some(pname) if !pname.is_empty() => {
            McpResponse::Success(format!("Set parent of '{}' to '{}'", name, pname))
        }
        _ => McpResponse::Success(format!("Unparented '{}'", name)),
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_query_entity(world: &World, name: String) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(transform) = world.get_local_transform(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no transform", name));
    };

    McpResponse::EntityInfo {
        name,
        position: [
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ],
        rotation: [
            transform.rotation.i,
            transform.rotation.j,
            transform.rotation.k,
            transform.rotation.w,
        ],
        scale: [transform.scale.x, transform.scale.y, transform.scale.z],
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_clear_scene(world: &mut World) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let entities_to_despawn: Vec<_> = world.resources.entity_names.values().copied().collect();
    let count = entities_to_despawn.len();
    for entity in entities_to_despawn {
        crate::ecs::world::commands::despawn_recursive_immediate(world, entity);
    }
    world.resources.entity_names.clear();
    McpResponse::Success(format!("Cleared {} entities from scene", count))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_atmosphere(world: &mut World, atmosphere: String) -> crate::mcp::McpResponse {
    use crate::ecs::graphics::resources::Atmosphere;
    use crate::mcp::McpResponse;

    let atmo = match atmosphere.to_lowercase().as_str() {
        "none" => Atmosphere::None,
        "sky" => Atmosphere::Sky,
        "cloudy" | "cloudy_sky" | "cloudysky" => Atmosphere::CloudySky,
        "space" => Atmosphere::Space,
        "nebula" => Atmosphere::Nebula,
        "sunset" => Atmosphere::Sunset,
        "hdr" => Atmosphere::Hdr,
        _ => {
            return McpResponse::Error(format!(
                "Unknown atmosphere '{}'. Valid: none, sky, cloudy_sky, space, nebula, sunset, hdr",
                atmosphere
            ));
        }
    };

    world.resources.graphics.atmosphere = atmo;
    McpResponse::Success(format!("Atmosphere set to {}", atmosphere))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_load_hdr(world: &mut World, path: String) -> crate::mcp::McpResponse {
    use crate::ecs::graphics::resources::Atmosphere;
    use crate::mcp::McpResponse;

    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return McpResponse::Error(format!("HDR file not found: {}", path));
    }

    crate::ecs::world::commands::load_hdr_skybox_from_path(world, path_buf);
    world.resources.graphics.atmosphere = Atmosphere::Hdr;
    McpResponse::Success(format!("Loading HDR skybox from {}", path))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
#[allow(clippy::too_many_arguments)]
fn mcp_spawn_light(
    world: &mut World,
    name: String,
    light_type: String,
    position: [f32; 3],
    color: Option<[f32; 3]>,
    intensity: Option<f32>,
    range: Option<f32>,
    inner_cone_angle: Option<f32>,
    outer_cone_angle: Option<f32>,
    cast_shadows: Option<bool>,
) -> crate::mcp::McpResponse {
    use crate::ecs::light::components::{Light, LightType};
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };
    use crate::mcp::McpResponse;

    if world.resources.entity_names.contains_key(&name) {
        return McpResponse::Error(format!("Entity '{}' already exists", name));
    }

    let light_type_enum = match light_type.to_lowercase().as_str() {
        "point" => LightType::Point,
        "spot" => LightType::Spot,
        "directional" => LightType::Directional,
        _ => return McpResponse::Error(format!("Unknown light type: {}", light_type)),
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world.set_name(entity, crate::ecs::name::components::Name(name.clone()));
    world.set_local_transform(
        entity,
        crate::ecs::transform::components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(position[0], position[1], position[2]),
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set_local_transform_dirty(
        entity,
        crate::ecs::transform::components::LocalTransformDirty,
    );
    world.set_global_transform(
        entity,
        crate::ecs::transform::components::GlobalTransform::default(),
    );

    let color_vec = color
        .map(|c| nalgebra_glm::Vec3::new(c[0], c[1], c[2]))
        .unwrap_or(nalgebra_glm::Vec3::new(1.0, 1.0, 1.0));

    world.set_light(
        entity,
        Light {
            light_type: light_type_enum,
            color: color_vec,
            intensity: intensity.unwrap_or(5.0),
            range: range.unwrap_or(100.0),
            inner_cone_angle: inner_cone_angle
                .map(|a| a.to_radians())
                .unwrap_or(std::f32::consts::PI / 6.0),
            outer_cone_angle: outer_cone_angle
                .map(|a| a.to_radians())
                .unwrap_or(std::f32::consts::PI / 4.0),
            cast_shadows: cast_shadows.unwrap_or(false),
            shadow_bias: 0.0005,
        },
    );

    world.resources.entity_names.insert(name.clone(), entity);
    McpResponse::Success(format!("Spawned {} light '{}'", light_type, name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
#[allow(clippy::too_many_arguments)]
fn mcp_set_light(
    world: &mut World,
    name: String,
    color: Option<[f32; 3]>,
    intensity: Option<f32>,
    range: Option<f32>,
    inner_cone_angle: Option<f32>,
    outer_cone_angle: Option<f32>,
    cast_shadows: Option<bool>,
    shadow_bias: Option<f32>,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(light) = world.get_light_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no light component", name));
    };

    if let Some(c) = color {
        light.color = nalgebra_glm::Vec3::new(c[0], c[1], c[2]);
    }
    if let Some(i) = intensity {
        light.intensity = i;
    }
    if let Some(r) = range {
        light.range = r;
    }
    if let Some(angle) = inner_cone_angle {
        light.inner_cone_angle = angle.to_radians();
    }
    if let Some(angle) = outer_cone_angle {
        light.outer_cone_angle = angle.to_radians();
    }
    if let Some(shadows) = cast_shadows {
        light.cast_shadows = shadows;
    }
    if let Some(bias) = shadow_bias {
        light.shadow_bias = bias;
    }

    McpResponse::Success(format!("Updated light '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_add_line(
    world: &mut World,
    start: [f32; 3],
    end: [f32; 3],
    color: Option<[f32; 4]>,
) -> crate::mcp::McpResponse {
    use crate::ecs::lines::components::{Line, Lines};
    use crate::ecs::world::{GLOBAL_TRANSFORM, LINES, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY};
    use crate::mcp::McpResponse;

    let lines_entity = world.query_entities(LINES).next().unwrap_or_else(|| {
        let entity = world.spawn_entities(
            LINES | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
            1,
        )[0];
        world.set_local_transform(
            entity,
            crate::ecs::transform::components::LocalTransform::default(),
        );
        world.set_local_transform_dirty(
            entity,
            crate::ecs::transform::components::LocalTransformDirty,
        );
        world.set_global_transform(
            entity,
            crate::ecs::transform::components::GlobalTransform::default(),
        );
        world.set_lines(entity, Lines::default());
        entity
    });

    let color_vec = color
        .map(|c| nalgebra_glm::Vec4::new(c[0], c[1], c[2], c[3]))
        .unwrap_or(nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0));

    if let Some(lines) = world.get_lines_mut(lines_entity) {
        lines.push(Line {
            start: nalgebra_glm::Vec3::new(start[0], start[1], start[2]),
            end: nalgebra_glm::Vec3::new(end[0], end[1], end[2]),
            color: color_vec,
        });
    }

    McpResponse::Success("Added line".to_string())
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_add_lines(
    world: &mut World,
    lines: Vec<crate::mcp::AddLineRequest>,
) -> crate::mcp::McpResponse {
    use crate::ecs::lines::components::{Line, Lines};
    use crate::ecs::world::{GLOBAL_TRANSFORM, LINES, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY};
    use crate::mcp::McpResponse;

    let lines_entity = world.query_entities(LINES).next().unwrap_or_else(|| {
        let entity = world.spawn_entities(
            LINES | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
            1,
        )[0];
        world.set_local_transform(
            entity,
            crate::ecs::transform::components::LocalTransform::default(),
        );
        world.set_local_transform_dirty(
            entity,
            crate::ecs::transform::components::LocalTransformDirty,
        );
        world.set_global_transform(
            entity,
            crate::ecs::transform::components::GlobalTransform::default(),
        );
        world.set_lines(entity, Lines::default());
        entity
    });

    let count = lines.len();
    if let Some(lines_component) = world.get_lines_mut(lines_entity) {
        for line in lines {
            let color = line
                .color
                .map(|c| nalgebra_glm::Vec4::new(c[0], c[1], c[2], c[3]))
                .unwrap_or(nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0));
            lines_component.push(Line {
                start: nalgebra_glm::Vec3::new(line.start[0], line.start[1], line.start[2]),
                end: nalgebra_glm::Vec3::new(line.end[0], line.end[1], line.end[2]),
                color,
            });
        }
    }

    McpResponse::Success(format!("Added {} lines", count))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_clear_lines(world: &mut World) -> crate::mcp::McpResponse {
    use crate::ecs::world::LINES;
    use crate::mcp::McpResponse;

    for entity in world.query_entities(LINES).collect::<Vec<_>>() {
        if let Some(lines) = world.get_lines_mut(entity) {
            lines.clear();
        }
    }

    McpResponse::Success("Cleared all lines".to_string())
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
#[allow(clippy::too_many_arguments)]
fn mcp_set_graphics(
    world: &mut World,
    bloom_enabled: Option<bool>,
    bloom_intensity: Option<f32>,
    ssao_enabled: Option<bool>,
    ssao_radius: Option<f32>,
    ssao_intensity: Option<f32>,
    ssao_bias: Option<f32>,
    fog_enabled: Option<bool>,
    fog_color: Option<[f32; 3]>,
    fog_start: Option<f32>,
    fog_end: Option<f32>,
    tonemap: Option<String>,
    gamma: Option<f32>,
    saturation: Option<f32>,
    brightness: Option<f32>,
    contrast: Option<f32>,
    show_grid: Option<bool>,
    clear_color: Option<[f32; 4]>,
    dof_enabled: Option<bool>,
    dof_focus_distance: Option<f32>,
    dof_focus_range: Option<f32>,
    dof_max_blur_radius: Option<f32>,
    unlit_mode: Option<bool>,
) -> crate::mcp::McpResponse {
    use crate::ecs::graphics::resources::{ColorGradingPreset, Fog, TonemapAlgorithm};
    use crate::mcp::McpResponse;

    let graphics = &mut world.resources.graphics;

    if let Some(enabled) = bloom_enabled {
        graphics.bloom_enabled = enabled;
    }
    if let Some(intensity) = bloom_intensity {
        graphics.bloom_intensity = intensity;
    }
    if let Some(enabled) = ssao_enabled {
        graphics.ssao_enabled = enabled;
    }
    if let Some(radius) = ssao_radius {
        graphics.ssao_radius = radius;
    }
    if let Some(intensity) = ssao_intensity {
        graphics.ssao_intensity = intensity;
    }
    if let Some(bias) = ssao_bias {
        graphics.ssao_bias = bias;
    }
    if let Some(enabled) = fog_enabled {
        if enabled {
            let fog = graphics.fog.get_or_insert(Fog::default());
            if let Some(color) = fog_color {
                fog.color = color;
            }
            if let Some(start) = fog_start {
                fog.start = start;
            }
            if let Some(end) = fog_end {
                fog.end = end;
            }
        } else {
            graphics.fog = None;
        }
    } else if let Some(fog) = graphics.fog.as_mut() {
        if let Some(color) = fog_color {
            fog.color = color;
        }
        if let Some(start) = fog_start {
            fog.start = start;
        }
        if let Some(end) = fog_end {
            fog.end = end;
        }
    }
    if let Some(algo) = tonemap {
        graphics.color_grading.tonemap_algorithm = match algo.to_lowercase().as_str() {
            "aces" => TonemapAlgorithm::Aces,
            "reinhard" => TonemapAlgorithm::Reinhard,
            "reinhard_extended" => TonemapAlgorithm::ReinhardExtended,
            "uncharted2" => TonemapAlgorithm::Uncharted2,
            "agx" => TonemapAlgorithm::AgX,
            "neutral" => TonemapAlgorithm::Neutral,
            "none" => TonemapAlgorithm::None,
            _ => TonemapAlgorithm::Aces,
        };
        graphics.color_grading.preset = ColorGradingPreset::Custom;
    }
    if let Some(g) = gamma {
        graphics.color_grading.gamma = g;
        graphics.color_grading.preset = ColorGradingPreset::Custom;
    }
    if let Some(s) = saturation {
        graphics.color_grading.saturation = s;
        graphics.color_grading.preset = ColorGradingPreset::Custom;
    }
    if let Some(b) = brightness {
        graphics.color_grading.brightness = b;
        graphics.color_grading.preset = ColorGradingPreset::Custom;
    }
    if let Some(c) = contrast {
        graphics.color_grading.contrast = c;
        graphics.color_grading.preset = ColorGradingPreset::Custom;
    }
    if let Some(grid) = show_grid {
        graphics.show_grid = grid;
    }
    if let Some(color) = clear_color {
        graphics.clear_color = color;
    }
    if let Some(enabled) = dof_enabled {
        graphics.depth_of_field.enabled = enabled;
    }
    if let Some(dist) = dof_focus_distance {
        graphics.depth_of_field.focus_distance = dist;
    }
    if let Some(range) = dof_focus_range {
        graphics.depth_of_field.focus_range = range;
    }
    if let Some(radius) = dof_max_blur_radius {
        graphics.depth_of_field.max_blur_radius = radius;
    }
    if let Some(unlit) = unlit_mode {
        graphics.unlit_mode = unlit;
    }

    McpResponse::Success("Graphics settings updated".to_string())
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
#[allow(clippy::too_many_arguments)]
fn mcp_spawn_water(
    world: &mut World,
    name: String,
    position: [f32; 3],
    scale: Option<[f32; 3]>,
    base_color: Option<[f32; 4]>,
    water_color: Option<[f32; 4]>,
    wave_height: Option<f32>,
    choppy: Option<f32>,
    speed: Option<f32>,
    frequency: Option<f32>,
) -> crate::mcp::McpResponse {
    use crate::ecs::mesh::components::create_subdivided_plane_mesh;
    use crate::ecs::prefab::resources::mesh_cache_insert;
    use crate::ecs::water::Water;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, RENDER_MESH, VISIBILITY,
        WATER,
    };
    use crate::mcp::McpResponse;

    if world.resources.entity_names.contains_key(&name) {
        return McpResponse::Error(format!("Entity '{}' already exists", name));
    }

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | VISIBILITY
            | WATER,
        1,
    )[0];

    let scale_vec = scale.unwrap_or([10.0, 1.0, 10.0]);
    world.set_name(entity, crate::ecs::name::components::Name(name.clone()));
    world.set_local_transform(
        entity,
        crate::ecs::transform::components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(position[0], position[1], position[2]),
            scale: nalgebra_glm::Vec3::new(scale_vec[0], scale_vec[1], scale_vec[2]),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.set_global_transform(
        entity,
        crate::ecs::transform::components::GlobalTransform::default(),
    );
    world.set_local_transform_dirty(
        entity,
        crate::ecs::transform::components::LocalTransformDirty,
    );
    world.set_visibility(
        entity,
        crate::ecs::visibility::components::Visibility { visible: true },
    );

    let mesh_name = "WaterPlane";
    if !world
        .resources
        .mesh_cache
        .registry
        .name_to_index
        .contains_key(mesh_name)
    {
        let mesh = create_subdivided_plane_mesh(20.0, 1);
        mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.to_string(), mesh);
    }
    world.set_render_mesh(
        entity,
        crate::ecs::mesh::components::RenderMesh::new(mesh_name),
    );

    let mut water = Water::default();
    if let Some(bc) = base_color {
        water.base_color = bc;
    }
    if let Some(wc) = water_color {
        water.water_color = wc;
    }
    if let Some(wh) = wave_height {
        water.wave_height = wh;
    }
    if let Some(c) = choppy {
        water.choppy = c;
    }
    if let Some(s) = speed {
        water.speed = s;
    }
    if let Some(f) = frequency {
        water.frequency = f;
    }
    world.set_water(entity, water);

    world.resources.entity_names.insert(name.clone(), entity);
    McpResponse::Success(format!("Spawned water plane '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
#[allow(clippy::too_many_arguments)]
fn mcp_spawn_hud_text(
    world: &mut World,
    name: String,
    text: String,
    position: Option<[f32; 2]>,
    anchor: Option<String>,
    font_size: Option<f32>,
    color: Option<[f32; 4]>,
    alignment: Option<String>,
    outline_width: Option<f32>,
    outline_color: Option<[f32; 4]>,
) -> crate::mcp::McpResponse {
    use crate::ecs::text::components::{HudAnchor, HudText, TextAlignment, TextProperties};
    use crate::ecs::world::{HUD_TEXT, NAME};
    use crate::mcp::McpResponse;

    if world.resources.entity_names.contains_key(&name) {
        return McpResponse::Error(format!("Entity '{}' already exists", name));
    }

    let text_index = world.resources.text_cache.add_text(text.clone());

    let anchor_enum = anchor
        .as_ref()
        .map(|a| match a.to_lowercase().as_str() {
            "top_left" => HudAnchor::TopLeft,
            "top_center" => HudAnchor::TopCenter,
            "top_right" => HudAnchor::TopRight,
            "center_left" => HudAnchor::CenterLeft,
            "center" => HudAnchor::Center,
            "center_right" => HudAnchor::CenterRight,
            "bottom_left" => HudAnchor::BottomLeft,
            "bottom_center" => HudAnchor::BottomCenter,
            "bottom_right" => HudAnchor::BottomRight,
            _ => HudAnchor::Center,
        })
        .unwrap_or(HudAnchor::Center);

    let text_alignment = alignment
        .as_ref()
        .map(|a| match a.to_lowercase().as_str() {
            "left" => TextAlignment::Left,
            "center" => TextAlignment::Center,
            "right" => TextAlignment::Right,
            _ => TextAlignment::Left,
        })
        .unwrap_or(TextAlignment::Left);

    let mut properties = TextProperties::default();
    if let Some(size) = font_size {
        properties.font_size = size;
    }
    if let Some(c) = color {
        properties.color = nalgebra_glm::Vec4::new(c[0], c[1], c[2], c[3]);
    }
    properties.alignment = text_alignment;
    if let Some(width) = outline_width {
        properties.outline_width = width;
    }
    if let Some(oc) = outline_color {
        properties.outline_color = nalgebra_glm::Vec4::new(oc[0], oc[1], oc[2], oc[3]);
    }

    let pos = position
        .map(|p| nalgebra_glm::Vec2::new(p[0], p[1]))
        .unwrap_or(nalgebra_glm::Vec2::zeros());

    let entity = world.spawn_entities(NAME | HUD_TEXT, 1)[0];
    world.set_name(entity, crate::ecs::name::components::Name(name.clone()));
    world.set_hud_text(
        entity,
        HudText::new(text_index)
            .with_properties(properties)
            .with_position(pos)
            .with_anchor(anchor_enum),
    );

    world.resources.entity_names.insert(name.clone(), entity);
    McpResponse::Success(format!("Spawned HUD text '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
#[allow(clippy::too_many_arguments)]
fn mcp_set_hud_text(
    world: &mut World,
    name: String,
    text: Option<String>,
    position: Option<[f32; 2]>,
    anchor: Option<String>,
    font_size: Option<f32>,
    color: Option<[f32; 4]>,
    alignment: Option<String>,
    outline_width: Option<f32>,
    outline_color: Option<[f32; 4]>,
) -> crate::mcp::McpResponse {
    use crate::ecs::text::components::{HudAnchor, TextAlignment};
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    if let Some(new_text) = text {
        let text_index = world.resources.text_cache.add_text(new_text);
        if let Some(hud_text) = world.get_hud_text_mut(entity) {
            hud_text.set_text_index(text_index);
        }
    }

    if let Some(hud_text) = world.get_hud_text_mut(entity) {
        if let Some(pos) = position {
            hud_text.set_position(nalgebra_glm::Vec2::new(pos[0], pos[1]));
        }
        if let Some(a) = anchor {
            let anchor_enum = match a.to_lowercase().as_str() {
                "top_left" => HudAnchor::TopLeft,
                "top_center" => HudAnchor::TopCenter,
                "top_right" => HudAnchor::TopRight,
                "center_left" => HudAnchor::CenterLeft,
                "center" => HudAnchor::Center,
                "center_right" => HudAnchor::CenterRight,
                "bottom_left" => HudAnchor::BottomLeft,
                "bottom_center" => HudAnchor::BottomCenter,
                "bottom_right" => HudAnchor::BottomRight,
                _ => HudAnchor::Center,
            };
            hud_text.set_anchor(anchor_enum);
        }
        if let Some(size) = font_size {
            hud_text.properties.font_size = size;
            hud_text.dirty = true;
        }
        if let Some(c) = color {
            hud_text.properties.color = nalgebra_glm::Vec4::new(c[0], c[1], c[2], c[3]);
            hud_text.dirty = true;
        }
        if let Some(a) = alignment {
            hud_text.properties.alignment = match a.to_lowercase().as_str() {
                "left" => TextAlignment::Left,
                "center" => TextAlignment::Center,
                "right" => TextAlignment::Right,
                _ => TextAlignment::Left,
            };
            hud_text.dirty = true;
        }
        if let Some(width) = outline_width {
            hud_text.properties.outline_width = width;
            hud_text.dirty = true;
        }
        if let Some(oc) = outline_color {
            hud_text.properties.outline_color = nalgebra_glm::Vec4::new(oc[0], oc[1], oc[2], oc[3]);
            hud_text.dirty = true;
        }
    } else {
        return McpResponse::Error(format!("Entity '{}' has no HUD text component", name));
    }

    McpResponse::Success(format!("Updated HUD text '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_camera(
    world: &mut World,
    position: Option<[f32; 3]>,
    target: Option<[f32; 3]>,
    fov: Option<f32>,
) -> crate::mcp::McpResponse {
    use crate::ecs::camera::components::Projection;
    use crate::mcp::McpResponse;

    let Some(camera_entity) = world.resources.active_camera else {
        return McpResponse::Error("No active camera".to_string());
    };

    if let Some(pos) = position
        && let Some(transform) = world.get_local_transform_mut(camera_entity)
    {
        transform.translation = nalgebra_glm::Vec3::new(pos[0], pos[1], pos[2]);
        crate::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
    }

    if let Some(target_pos) = target
        && let Some(transform) = world.get_local_transform_mut(camera_entity)
    {
        let eye = transform.translation;
        let target = nalgebra_glm::Vec3::new(target_pos[0], target_pos[1], target_pos[2]);
        let up = nalgebra_glm::Vec3::new(0.0, 1.0, 0.0);

        let view = nalgebra_glm::look_at(&eye, &target, &up);
        let rotation_matrix = view.fixed_view::<3, 3>(0, 0).transpose();
        transform.rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
        crate::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
    }

    if let Some(fov_degrees) = fov
        && let Some(camera) = world.get_camera_mut(camera_entity)
        && let Projection::Perspective(ref mut perspective) = camera.projection
    {
        perspective.y_fov_rad = fov_degrees.to_radians();
    }

    McpResponse::Success("Camera updated".to_string())
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_script(
    world: &mut World,
    name: String,
    source: String,
    enabled: bool,
) -> crate::mcp::McpResponse {
    use crate::ecs::script::components::{Script, ScriptSource};
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let script = Script {
        source: ScriptSource::Embedded { source },
        enabled,
    };

    world.set_script(entity, script);
    McpResponse::Success(format!("Script set on '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_remove_script(world: &mut World, name: String) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    world.remove_script(entity);
    McpResponse::Success(format!("Script removed from '{}'", name))
}

#[cfg(all(feature = "mcp", feature = "scripting", not(target_arch = "wasm32")))]
fn mcp_set_game_state(
    world: &mut World,
    values: std::collections::HashMap<String, f64>,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    for (key, value) in values {
        world.resources.script_runtime.game_state.insert(key, value);
    }
    McpResponse::Success("Game state updated".to_string())
}

#[cfg(all(
    feature = "mcp",
    not(feature = "scripting"),
    not(target_arch = "wasm32")
))]
fn mcp_set_game_state(
    _world: &mut World,
    _values: std::collections::HashMap<String, f64>,
) -> crate::mcp::McpResponse {
    crate::mcp::McpResponse::Error("Scripting feature not enabled".to_string())
}

#[cfg(all(feature = "mcp", feature = "scripting", not(target_arch = "wasm32")))]
fn mcp_get_game_state(world: &mut World, keys: Option<Vec<String>>) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let state = match keys {
        Some(keys) => {
            let mut result = std::collections::HashMap::new();
            for key in keys {
                if let Some(&value) = world.resources.script_runtime.game_state.get(&key) {
                    result.insert(key, value);
                }
            }
            result
        }
        None => world.resources.script_runtime.game_state.clone(),
    };

    McpResponse::GameState(state)
}

#[cfg(all(
    feature = "mcp",
    not(feature = "scripting"),
    not(target_arch = "wasm32")
))]
fn mcp_get_game_state(_world: &mut World, _keys: Option<Vec<String>>) -> crate::mcp::McpResponse {
    crate::mcp::McpResponse::Error("Scripting feature not enabled".to_string())
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_visibility(world: &mut World, name: String, visible: bool) -> crate::mcp::McpResponse {
    use crate::ecs::visibility::components::Visibility;
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    world.set_visibility(entity, Visibility { visible });
    McpResponse::Success(format!("Visibility set to {} for '{}'", visible, name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_play_animation(
    world: &mut World,
    name: String,
    clip: String,
    looping: bool,
    speed: f32,
    blend_duration: f32,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(player) = world.get_animation_player_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no AnimationPlayer", name));
    };

    let clip_index = if let Ok(index) = clip.parse::<usize>() {
        index
    } else {
        match player.clips.iter().position(|c| c.name == clip) {
            Some(index) => index,
            None => return McpResponse::Error(format!("Animation clip '{}' not found", clip)),
        }
    };

    player.looping = looping;
    player.speed = speed;

    if blend_duration > 0.0 {
        player.blend_to(clip_index, blend_duration);
    } else {
        player.play(clip_index);
    }

    McpResponse::Success(format!("Playing animation '{}' on '{}'", clip, name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_stop_animation(world: &mut World, name: String) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(player) = world.get_animation_player_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no AnimationPlayer", name));
    };

    player.stop();
    McpResponse::Success(format!("Animation stopped on '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_list_animations(world: &mut World, name: String) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(player) = world.get_animation_player(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no AnimationPlayer", name));
    };

    let names: Vec<String> = player.clips.iter().map(|c| c.name.clone()).collect();
    McpResponse::AnimationList(names)
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_spawn_particles(
    world: &mut World,
    name: String,
    position: [f32; 3],
    preset: String,
    color: Option<[f32; 3]>,
    particle_count: Option<u32>,
    enabled: bool,
) -> crate::mcp::McpResponse {
    use crate::ecs::particles::components::ParticleEmitter;
    use crate::ecs::transform::components::LocalTransform;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, PARTICLE_EMITTER,
    };
    use crate::mcp::McpResponse;

    let pos = nalgebra_glm::Vec3::new(position[0], position[1], position[2]);
    let color_vec = color.map(|c| nalgebra_glm::Vec3::new(c[0], c[1], c[2]));
    let count = particle_count.unwrap_or(200);

    let mut emitter = match preset.to_lowercase().as_str() {
        "fire" => ParticleEmitter::fire(nalgebra_glm::Vec3::zeros()),
        "smoke" => ParticleEmitter::smoke(nalgebra_glm::Vec3::zeros()),
        "sparks" => ParticleEmitter::sparks(nalgebra_glm::Vec3::zeros()),
        "firework_explosion" | "firework" | "explosion" => {
            let c = color_vec.unwrap_or(nalgebra_glm::Vec3::new(1.0, 0.5, 0.2));
            ParticleEmitter::firework_explosion(nalgebra_glm::Vec3::zeros(), c, count)
        }
        "firework_willow" | "willow" => {
            let c = color_vec.unwrap_or(nalgebra_glm::Vec3::new(0.8, 0.9, 1.0));
            ParticleEmitter::firework_willow(nalgebra_glm::Vec3::zeros(), c, count)
        }
        "firework_chrysanthemum" | "chrysanthemum" => {
            let c = color_vec.unwrap_or(nalgebra_glm::Vec3::new(1.0, 0.3, 0.3));
            ParticleEmitter::firework_chrysanthemum(nalgebra_glm::Vec3::zeros(), c, count)
        }
        "firework_ring" | "ring" => {
            let c = color_vec.unwrap_or(nalgebra_glm::Vec3::new(0.3, 0.8, 1.0));
            ParticleEmitter::firework_ring(nalgebra_glm::Vec3::zeros(), c, count)
        }
        "flash" | "flash_burst" => ParticleEmitter::flash_burst(nalgebra_glm::Vec3::zeros()),
        _ => return McpResponse::Error(format!("Unknown particle preset '{}'", preset)),
    };

    emitter.enabled = enabled;

    let entity = world
        .spawn_entities(
            NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | PARTICLE_EMITTER,
            1,
        )
        .into_iter()
        .next()
        .unwrap();
    world.set_local_transform(entity, LocalTransform::from_translation(pos));
    world.set_particle_emitter(entity, emitter);
    world.set_name(entity, crate::ecs::name::components::Name(name.clone()));
    world.resources.entity_names.insert(name.clone(), entity);

    McpResponse::Success(format!("Spawned particle emitter '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_particles(
    world: &mut World,
    name: String,
    enabled: Option<bool>,
    spawn_rate: Option<f32>,
    emissive_strength: Option<f32>,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(emitter) = world.get_particle_emitter_mut(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no particle emitter", name));
    };

    if let Some(e) = enabled {
        emitter.enabled = e;
    }
    if let Some(rate) = spawn_rate {
        emitter.spawn_rate = rate;
    }
    if let Some(strength) = emissive_strength {
        emitter.emissive_strength = strength;
    }

    McpResponse::Success(format!("Particle emitter '{}' updated", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_add_rigid_body(
    world: &mut World,
    name: String,
    body_type: String,
    mass: Option<f32>,
    linear_velocity: Option<[f32; 3]>,
    angular_velocity: Option<[f32; 3]>,
) -> crate::mcp::McpResponse {
    use crate::ecs::physics::components::RigidBodyComponent;
    use crate::ecs::physics::types::RigidBodyType;
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let rb_type = match body_type.to_lowercase().as_str() {
        "dynamic" => RigidBodyType::Dynamic,
        "kinematic" | "kinematic_position" => RigidBodyType::KinematicPositionBased,
        "kinematic_velocity" => RigidBodyType::KinematicVelocityBased,
        "static" | "fixed" => RigidBodyType::Fixed,
        _ => {
            return McpResponse::Error(format!(
                "Unknown body type '{}'. Use: dynamic, kinematic, static",
                body_type
            ));
        }
    };

    let mut rb = RigidBodyComponent {
        body_type: rb_type,
        mass: mass.unwrap_or(1.0),
        ..Default::default()
    };

    if let Some(lv) = linear_velocity {
        rb.linvel = lv;
    }
    if let Some(av) = angular_velocity {
        rb.angvel = av;
    }

    if let Some(transform) = world.get_local_transform(entity) {
        rb.translation = [
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ];
        rb.rotation = [
            transform.rotation.i,
            transform.rotation.j,
            transform.rotation.k,
            transform.rotation.w,
        ];
    }

    world.set_rigid_body(entity, rb);
    McpResponse::Success(format!("Rigid body added to '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_add_collider(
    world: &mut World,
    name: String,
    shape: String,
    size: Vec<f32>,
    friction: Option<f32>,
    restitution: Option<f32>,
    is_sensor: Option<bool>,
) -> crate::mcp::McpResponse {
    use crate::ecs::physics::components::{ColliderComponent, ColliderShape};
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let collider_shape = match shape.to_lowercase().as_str() {
        "ball" | "sphere" => {
            let radius = size.first().copied().unwrap_or(0.5);
            ColliderShape::Ball { radius }
        }
        "cuboid" | "box" => {
            let hx = size.first().copied().unwrap_or(0.5);
            let hy = size.get(1).copied().unwrap_or(hx);
            let hz = size.get(2).copied().unwrap_or(hx);
            ColliderShape::Cuboid { hx, hy, hz }
        }
        "capsule" => {
            let half_height = size.first().copied().unwrap_or(0.5);
            let radius = size.get(1).copied().unwrap_or(0.25);
            ColliderShape::Capsule {
                half_height,
                radius,
            }
        }
        "cylinder" => {
            let half_height = size.first().copied().unwrap_or(0.5);
            let radius = size.get(1).copied().unwrap_or(0.25);
            ColliderShape::Cylinder {
                half_height,
                radius,
            }
        }
        _ => {
            return McpResponse::Error(format!(
                "Unknown collider shape '{}'. Use: ball, cuboid, capsule, cylinder",
                shape
            ));
        }
    };

    let collider = ColliderComponent {
        shape: collider_shape,
        friction: friction.unwrap_or(0.5),
        restitution: restitution.unwrap_or(0.0),
        is_sensor: is_sensor.unwrap_or(false),
        ..Default::default()
    };

    world.set_collider(entity, collider);
    McpResponse::Success(format!("Collider added to '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_apply_impulse(
    world: &mut World,
    name: String,
    impulse: [f32; 3],
    torque: bool,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(rb) = world.get_rigid_body(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no rigid body", name));
    };

    let Some(handle) = rb.handle else {
        return McpResponse::Error(format!(
            "Rigid body '{}' not yet initialized in physics world",
            name
        ));
    };

    let impulse_vec = rapier3d::prelude::vector![impulse[0], impulse[1], impulse[2]];

    if let Some(body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    {
        if torque {
            body.apply_torque_impulse(impulse_vec, true);
        } else {
            body.apply_impulse(impulse_vec, true);
        }
    }

    McpResponse::Success(format!("Impulse applied to '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_apply_force(
    world: &mut World,
    name: String,
    force: [f32; 3],
    torque: bool,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(rb) = world.get_rigid_body(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no rigid body", name));
    };

    let Some(handle) = rb.handle else {
        return McpResponse::Error(format!(
            "Rigid body '{}' not yet initialized in physics world",
            name
        ));
    };

    let force_vec = rapier3d::prelude::vector![force[0], force[1], force[2]];

    if let Some(body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    {
        if torque {
            body.add_torque(force_vec, true);
        } else {
            body.add_force(force_vec, true);
        }
    }

    McpResponse::Success(format!("Force applied to '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_velocity(
    world: &mut World,
    name: String,
    linear: Option<[f32; 3]>,
    angular: Option<[f32; 3]>,
) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(rb) = world.get_rigid_body(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no rigid body", name));
    };

    let Some(handle) = rb.handle else {
        return McpResponse::Error(format!(
            "Rigid body '{}' not yet initialized in physics world",
            name
        ));
    };

    if let Some(body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    {
        if let Some(lv) = linear {
            body.set_linvel(rapier3d::prelude::vector![lv[0], lv[1], lv[2]], true);
        }
        if let Some(av) = angular {
            body.set_angvel(rapier3d::prelude::vector![av[0], av[1], av[2]], true);
        }
    }

    McpResponse::Success(format!("Velocity set on '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_spawn_3d_text(
    world: &mut World,
    name: String,
    text: String,
    position: [f32; 3],
    font_size: f32,
    color: [f32; 4],
    billboard: bool,
) -> crate::mcp::McpResponse {
    use crate::ecs::text::components::Text;
    use crate::ecs::transform::components::LocalTransform;
    use crate::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, TEXT};
    use crate::mcp::McpResponse;

    let text_index = world.resources.text_cache.add_text(&text);

    let entity = world
        .spawn_entities(
            NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT,
            1,
        )
        .into_iter()
        .next()
        .unwrap();

    let pos = nalgebra_glm::Vec3::new(position[0], position[1], position[2]);
    world.set_local_transform(entity, LocalTransform::from_translation(pos));

    let mut text_component = Text::new(text_index);
    text_component.billboard = billboard;
    text_component.properties.font_size = font_size;
    text_component.properties.color =
        nalgebra_glm::Vec4::new(color[0], color[1], color[2], color[3]);
    world.set_text(entity, text_component);
    world.set_name(entity, crate::ecs::name::components::Name(name.clone()));
    world.resources.entity_names.insert(name.clone(), entity);

    McpResponse::Success(format!("Spawned 3D text '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_spawn_decal(
    world: &mut World,
    name: String,
    texture: String,
    position: [f32; 3],
    size: [f32; 3],
    rotation: [f32; 3],
) -> crate::mcp::McpResponse {
    use crate::ecs::decal::components::Decal;
    use crate::ecs::transform::components::LocalTransform;
    use crate::ecs::world::{
        DECAL, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };
    use crate::mcp::McpResponse;

    let entity = world
        .spawn_entities(
            NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | DECAL,
            1,
        )
        .into_iter()
        .next()
        .unwrap();

    let pos = nalgebra_glm::Vec3::new(position[0], position[1], position[2]);
    let quat = nalgebra_glm::quat_identity();
    let quat =
        nalgebra_glm::quat_rotate(&quat, rotation[0], &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0));
    let quat =
        nalgebra_glm::quat_rotate(&quat, rotation[1], &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0));
    let quat =
        nalgebra_glm::quat_rotate(&quat, rotation[2], &nalgebra_glm::Vec3::new(0.0, 0.0, 1.0));

    let mut transform = LocalTransform::from_translation(pos);
    transform.rotation = quat;
    world.set_local_transform(entity, transform);

    let decal = Decal {
        texture: Some(texture),
        size: nalgebra_glm::Vec2::new(size[0], size[1]),
        depth: size[2],
        ..Default::default()
    };
    world.set_decal(entity, decal);
    world.set_name(entity, crate::ecs::name::components::Name(name.clone()));
    world.resources.entity_names.insert(name.clone(), entity);

    McpResponse::Success(format!("Spawned decal '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
struct SetMaterialParams {
    name: String,
    base_color: Option<[f32; 4]>,
    emissive: Option<[f32; 3]>,
    roughness: Option<f32>,
    metallic: Option<f32>,
    alpha_mode: Option<String>,
    alpha_cutoff: Option<f32>,
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_material(world: &mut World, params: SetMaterialParams) -> crate::mcp::McpResponse {
    let SetMaterialParams {
        name,
        base_color,
        emissive,
        roughness,
        metallic,
        alpha_mode,
        alpha_cutoff,
    } = params;
    use crate::ecs::generational_registry::registry_entry_by_name_mut;
    use crate::ecs::material::components::AlphaMode;
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    let Some(mat_ref) = world.get_material_ref(entity) else {
        return McpResponse::Error(format!("Entity '{}' has no material", name));
    };

    let material_name = mat_ref.name.clone();
    let Some(material) = registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        &material_name,
    ) else {
        return McpResponse::Error(format!(
            "Material '{}' not found for '{}'",
            material_name, name
        ));
    };

    if let Some(color) = base_color {
        material.base_color = color;
    }
    if let Some(em) = emissive {
        material.emissive_factor = em;
    }
    if let Some(r) = roughness {
        material.roughness = r;
    }
    if let Some(m) = metallic {
        material.metallic = m;
    }
    if let Some(mode) = alpha_mode {
        material.alpha_mode = match mode.to_lowercase().as_str() {
            "opaque" => AlphaMode::Opaque,
            "mask" => AlphaMode::Mask,
            "blend" => AlphaMode::Blend,
            _ => material.alpha_mode,
        };
    }
    if let Some(cutoff) = alpha_cutoff {
        material.alpha_cutoff = cutoff;
    }

    McpResponse::Success(format!("Material updated for '{}'", name))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_set_casts_shadow(
    world: &mut World,
    name: String,
    casts_shadow: bool,
) -> crate::mcp::McpResponse {
    use crate::ecs::shadow::components::CastsShadow;
    use crate::mcp::McpResponse;

    let Some(&entity) = world.resources.entity_names.get(&name) else {
        return McpResponse::Error(format!("Entity '{}' not found", name));
    };

    if casts_shadow {
        world.set_casts_shadow(entity, CastsShadow);
    } else {
        world.remove_casts_shadow(entity);
    }

    McpResponse::Success(format!(
        "Shadow casting set to {} for '{}'",
        casts_shadow, name
    ))
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_get_input(world: &mut World) -> crate::mcp::McpResponse {
    use crate::ecs::input::resources::MouseState;
    use crate::mcp::McpResponse;
    use winit::event::ElementState;

    let pressed_keys: Vec<String> = world
        .resources
        .input
        .keyboard
        .keystates
        .iter()
        .filter(|(_, state)| **state == ElementState::Pressed)
        .map(|(key, _)| format!("{:?}", key))
        .collect();

    let mouse_position = [
        world.resources.input.mouse.position.x,
        world.resources.input.mouse.position.y,
    ];

    let mouse_state = world.resources.input.mouse.state;
    let mouse_buttons = [
        mouse_state.contains(MouseState::LEFT_CLICKED),
        mouse_state.contains(MouseState::MIDDLE_CLICKED),
        mouse_state.contains(MouseState::RIGHT_CLICKED),
    ];

    McpResponse::InputState {
        pressed_keys,
        mouse_position,
        mouse_buttons,
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn mcp_get_time(world: &mut World) -> crate::mcp::McpResponse {
    use crate::mcp::McpResponse;

    let elapsed_time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    McpResponse::TimeInfo {
        delta_time: world.resources.window.timing.delta_time,
        elapsed_time,
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn process_mcp_commands(world: &mut World) {
    use crate::ecs::world::NAME;
    use crate::mcp::{McpCommand, McpResponse};

    let commands: Vec<McpCommand> = {
        let mut queue = world.resources.mcp_command_queue.write().unwrap();
        queue.drain(..).collect()
    };

    for cmd in commands {
        let response = match cmd {
            McpCommand::ListEntities => {
                let names: Vec<String> = world.resources.entity_names.keys().cloned().collect();
                McpResponse::EntityList(names)
            }
            McpCommand::QueryEntity { name } => mcp_query_entity(world, name),
            McpCommand::SpawnEntity {
                name,
                mesh,
                position,
                scale,
                color,
                emissive,
                parent,
                alpha,
            } => mcp_spawn_entity(
                world,
                SpawnEntityParams {
                    name,
                    mesh,
                    position,
                    scale,
                    color,
                    emissive,
                    parent_name: parent,
                    alpha_mode: alpha,
                },
            ),
            McpCommand::DespawnEntity { name } => mcp_despawn_entity(world, name),
            McpCommand::SetPosition { name, position } => mcp_set_position(world, name, position),
            McpCommand::SetRotation { name, rotation } => mcp_set_rotation(world, name, rotation),
            McpCommand::SetScale { name, scale } => mcp_set_scale(world, name, scale),
            McpCommand::SetMaterialColor { name, color } => {
                mcp_set_material_color(world, name, color)
            }
            McpCommand::SetEmissive { name, emissive } => mcp_set_emissive(world, name, emissive),
            McpCommand::SetParent { name, parent } => mcp_set_parent(world, name, parent),
            McpCommand::LoadAsset { path, asset_name } => {
                use crate::ecs::prefab::resources::{
                    CachedPrefab, mesh_cache_insert, prefab_cache_insert,
                };
                use crate::ecs::world::commands::WorldCommand;

                let extension = std::path::Path::new(&path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());

                let process_result =
                    |world: &mut World,
                     prefabs: Vec<crate::ecs::prefab::Prefab>,
                     meshes: std::collections::HashMap<
                        String,
                        crate::ecs::mesh::components::Mesh,
                    >,
                     textures: std::collections::HashMap<String, (Vec<u8>, u32, u32)>,
                     animations: Vec<crate::ecs::animation::components::AnimationClip>,
                     skins: Vec<crate::ecs::prefab::GltfSkin>,
                     asset_name: &str,
                     path: &str| {
                        for (name, (rgba_data, width, height)) in textures {
                            world.queue_command(WorldCommand::LoadTexture {
                                name,
                                rgba_data,
                                width,
                                height,
                            });
                        }

                        for (name, mesh) in meshes {
                            mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                        }

                        let prefab_count = prefabs.len();
                        for (index, prefab) in prefabs.into_iter().enumerate() {
                            let name = if prefab_count == 1 {
                                asset_name.to_string()
                            } else {
                                format!("{}_{}", asset_name, index)
                            };
                            prefab_cache_insert(
                                &mut world.resources.prefab_cache,
                                name,
                                CachedPrefab {
                                    prefab,
                                    animations: animations.clone(),
                                    skins: skins.clone(),
                                    source_path: Some(path.to_string()),
                                },
                            );
                        }

                        McpResponse::Success(format!(
                            "Loaded asset '{}' from '{}' ({} prefab(s))",
                            asset_name, path, prefab_count
                        ))
                    };

                match extension.as_deref() {
                    Some("glb") | Some("gltf") => match std::fs::read(&path) {
                        Ok(bytes) => match crate::ecs::prefab::import_gltf_from_bytes(&bytes) {
                            Ok(result) => process_result(
                                world,
                                result.prefabs,
                                result.meshes,
                                result.textures,
                                result.animations,
                                result.skins,
                                &asset_name,
                                &path,
                            ),
                            Err(e) => {
                                McpResponse::Error(format!("Failed to load '{}': {}", path, e))
                            }
                        },
                        Err(e) => McpResponse::Error(format!("Failed to read '{}': {}", path, e)),
                    },
                    #[cfg(feature = "fbx")]
                    Some("fbx") => {
                        match crate::ecs::prefab::import_fbx_from_path(std::path::Path::new(&path))
                        {
                            Ok(result) => process_result(
                                world,
                                result.prefabs,
                                result.meshes,
                                result.textures,
                                result.animations,
                                result.skins,
                                &asset_name,
                                &path,
                            ),
                            Err(e) => {
                                McpResponse::Error(format!("Failed to load '{}': {}", path, e))
                            }
                        }
                    }
                    _ => McpResponse::Error(
                        "Unsupported file extension. Supported: .glb, .gltf, .fbx".to_string(),
                    ),
                }
            }
            McpCommand::SpawnPrefab {
                asset_name,
                entity_name,
                position,
                scale,
            } => {
                use crate::ecs::prefab::resources::prefab_cache_get;
                use crate::ecs::prefab::spawn_prefab_with_skins;

                if world.resources.entity_names.contains_key(&entity_name) {
                    McpResponse::Error(format!("Entity '{}' already exists", entity_name))
                } else if let Some(cached) =
                    prefab_cache_get(&world.resources.prefab_cache, &asset_name)
                {
                    let prefab = cached.prefab.clone();
                    let animations = cached.animations.clone();
                    let skins = cached.skins.clone();
                    let pos = nalgebra_glm::Vec3::new(position[0], position[1], position[2]);

                    let entity = spawn_prefab_with_skins(world, &prefab, &animations, &skins, pos);

                    if scale != [1.0, 1.0, 1.0]
                        && let Some(transform) = world.get_local_transform_mut(entity)
                    {
                        transform.scale = nalgebra_glm::Vec3::new(scale[0], scale[1], scale[2]);
                        crate::ecs::transform::systems::mark_local_transform_dirty(world, entity);
                    }

                    world.add_components(entity, NAME);
                    world.set_name(
                        entity,
                        crate::ecs::name::components::Name(entity_name.clone()),
                    );
                    world
                        .resources
                        .entity_names
                        .insert(entity_name.clone(), entity);

                    McpResponse::Success(format!(
                        "Spawned prefab '{}' as entity '{}' at {:?}",
                        asset_name, entity_name, position
                    ))
                } else {
                    McpResponse::Error(format!(
                        "Asset '{}' not found. Use load_asset first.",
                        asset_name
                    ))
                }
            }
            McpCommand::ListLoadedAssets => {
                use crate::ecs::prefab::resources::prefab_cache_names;
                let names: Vec<String> = prefab_cache_names(&world.resources.prefab_cache)
                    .cloned()
                    .collect();
                McpResponse::AssetList(names)
            }
            McpCommand::ClearScene => mcp_clear_scene(world),
            McpCommand::SetAtmosphere { atmosphere } => mcp_set_atmosphere(world, atmosphere),
            McpCommand::LoadHdr { path } => mcp_load_hdr(world, path),
            McpCommand::SpawnLight {
                name,
                light_type,
                position,
                color,
                intensity,
                range,
                inner_cone_angle,
                outer_cone_angle,
                cast_shadows,
            } => mcp_spawn_light(
                world,
                name,
                light_type,
                position,
                color,
                intensity,
                range,
                inner_cone_angle,
                outer_cone_angle,
                cast_shadows,
            ),
            McpCommand::SetLight {
                name,
                color,
                intensity,
                range,
                inner_cone_angle,
                outer_cone_angle,
                cast_shadows,
                shadow_bias,
            } => mcp_set_light(
                world,
                name,
                color,
                intensity,
                range,
                inner_cone_angle,
                outer_cone_angle,
                cast_shadows,
                shadow_bias,
            ),
            McpCommand::AddLine { start, end, color } => mcp_add_line(world, start, end, color),
            McpCommand::AddLines { lines } => mcp_add_lines(world, lines),
            McpCommand::ClearLines => mcp_clear_lines(world),
            McpCommand::SetGraphics {
                bloom_enabled,
                bloom_intensity,
                ssao_enabled,
                ssao_radius,
                ssao_intensity,
                ssao_bias,
                fog_enabled,
                fog_color,
                fog_start,
                fog_end,
                tonemap,
                gamma,
                saturation,
                brightness,
                contrast,
                show_grid,
                clear_color,
                dof_enabled,
                dof_focus_distance,
                dof_focus_range,
                dof_max_blur_radius,
                unlit_mode,
            } => mcp_set_graphics(
                world,
                bloom_enabled,
                bloom_intensity,
                ssao_enabled,
                ssao_radius,
                ssao_intensity,
                ssao_bias,
                fog_enabled,
                fog_color,
                fog_start,
                fog_end,
                tonemap,
                gamma,
                saturation,
                brightness,
                contrast,
                show_grid,
                clear_color,
                dof_enabled,
                dof_focus_distance,
                dof_focus_range,
                dof_max_blur_radius,
                unlit_mode,
            ),
            McpCommand::SpawnWater {
                name,
                position,
                scale,
                base_color,
                water_color,
                wave_height,
                choppy,
                speed,
                frequency,
            } => mcp_spawn_water(
                world,
                name,
                position,
                scale,
                base_color,
                water_color,
                wave_height,
                choppy,
                speed,
                frequency,
            ),
            McpCommand::SpawnHudText {
                name,
                text,
                position,
                anchor,
                font_size,
                color,
                alignment,
                outline_width,
                outline_color,
            } => mcp_spawn_hud_text(
                world,
                name,
                text,
                position,
                anchor,
                font_size,
                color,
                alignment,
                outline_width,
                outline_color,
            ),
            McpCommand::SetHudText {
                name,
                text,
                position,
                anchor,
                font_size,
                color,
                alignment,
                outline_width,
                outline_color,
            } => mcp_set_hud_text(
                world,
                name,
                text,
                position,
                anchor,
                font_size,
                color,
                alignment,
                outline_width,
                outline_color,
            ),
            McpCommand::SetCamera {
                position,
                target,
                fov,
            } => mcp_set_camera(world, position, target, fov),
            McpCommand::SetScript {
                name,
                source,
                enabled,
            } => mcp_set_script(world, name, source, enabled),
            McpCommand::RemoveScript { name } => mcp_remove_script(world, name),
            McpCommand::SetGameState { values } => mcp_set_game_state(world, values),
            McpCommand::GetGameState { keys } => mcp_get_game_state(world, keys),
            McpCommand::SetVisibility { name, visible } => mcp_set_visibility(world, name, visible),
            McpCommand::PlayAnimation {
                name,
                clip,
                looping,
                speed,
                blend_duration,
            } => mcp_play_animation(world, name, clip, looping, speed, blend_duration),
            McpCommand::StopAnimation { name } => mcp_stop_animation(world, name),
            McpCommand::ListAnimations { name } => mcp_list_animations(world, name),
            McpCommand::SpawnParticles {
                name,
                position,
                preset,
                color,
                particle_count,
                enabled,
            } => mcp_spawn_particles(
                world,
                name,
                position,
                preset,
                color,
                particle_count,
                enabled,
            ),
            McpCommand::SetParticles {
                name,
                enabled,
                spawn_rate,
                emissive_strength,
            } => mcp_set_particles(world, name, enabled, spawn_rate, emissive_strength),
            McpCommand::AddRigidBody {
                name,
                body_type,
                mass,
                linear_velocity,
                angular_velocity,
            } => mcp_add_rigid_body(
                world,
                name,
                body_type,
                mass,
                linear_velocity,
                angular_velocity,
            ),
            McpCommand::AddCollider {
                name,
                shape,
                size,
                friction,
                restitution,
                is_sensor,
            } => mcp_add_collider(world, name, shape, size, friction, restitution, is_sensor),
            McpCommand::ApplyImpulse {
                name,
                impulse,
                torque,
            } => mcp_apply_impulse(world, name, impulse, torque),
            McpCommand::ApplyForce {
                name,
                force,
                torque,
            } => mcp_apply_force(world, name, force, torque),
            McpCommand::SetVelocity {
                name,
                linear,
                angular,
            } => mcp_set_velocity(world, name, linear, angular),
            McpCommand::Spawn3dText {
                name,
                text,
                position,
                font_size,
                color,
                billboard,
            } => mcp_spawn_3d_text(world, name, text, position, font_size, color, billboard),
            McpCommand::SpawnDecal {
                name,
                texture,
                position,
                size,
                rotation,
            } => mcp_spawn_decal(world, name, texture, position, size, rotation),
            McpCommand::SetMaterial {
                name,
                base_color,
                emissive,
                roughness,
                metallic,
                alpha_mode,
                alpha_cutoff,
            } => mcp_set_material(
                world,
                SetMaterialParams {
                    name,
                    base_color,
                    emissive,
                    roughness,
                    metallic,
                    alpha_mode,
                    alpha_cutoff,
                },
            ),
            McpCommand::SetCastsShadow { name, casts_shadow } => {
                mcp_set_casts_shadow(world, name, casts_shadow)
            }
            McpCommand::GetInput => mcp_get_input(world),
            McpCommand::GetTime => mcp_get_time(world),
            McpCommand::Batch(operations) => {
                let results = process_batch_operations(world, operations);
                McpResponse::BatchResult(results)
            }
            McpCommand::RunCommands(commands) => {
                let results = process_text_commands(world, &commands);
                McpResponse::BatchResult(results)
            }
        };

        let mut response_queue = world.resources.mcp_response_queue.write().unwrap();
        *response_queue = Some(response);
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn process_batch_operations(
    world: &mut World,
    operations: Vec<crate::mcp::BatchOperation>,
) -> Vec<crate::mcp::McpResponse> {
    use crate::mcp::McpResponse;

    let mut results = Vec::with_capacity(operations.len());

    for operation in operations {
        let response = match operation.operation.to_lowercase().as_str() {
            "spawn_entity" => {
                let (Some(name), Some(mesh)) = (operation.name, operation.mesh) else {
                    results.push(McpResponse::Error(
                        "spawn_entity requires 'name' and 'mesh' fields".to_string(),
                    ));
                    continue;
                };
                let position = operation.position.unwrap_or([0.0, 0.0, 0.0]);
                let scale = operation.scale.unwrap_or([1.0, 1.0, 1.0]);
                mcp_spawn_entity(
                    world,
                    SpawnEntityParams {
                        name,
                        mesh,
                        position,
                        scale,
                        color: operation.color,
                        emissive: operation.emissive,
                        parent_name: operation.parent,
                        alpha_mode: operation.alpha,
                    },
                )
            }
            "despawn_entity" => {
                let Some(name) = operation.name else {
                    results.push(McpResponse::Error(
                        "despawn_entity requires 'name' field".to_string(),
                    ));
                    continue;
                };
                mcp_despawn_entity(world, name)
            }
            "set_position" => {
                let (Some(name), Some(position)) = (operation.name, operation.position) else {
                    results.push(McpResponse::Error(
                        "set_position requires 'name' and 'position' fields".to_string(),
                    ));
                    continue;
                };
                mcp_set_position(world, name, position)
            }
            "set_rotation" => {
                let (Some(name), Some(rotation)) = (operation.name, operation.rotation) else {
                    results.push(McpResponse::Error(
                        "set_rotation requires 'name' and 'rotation' fields".to_string(),
                    ));
                    continue;
                };
                mcp_set_rotation(world, name, rotation)
            }
            "set_scale" => {
                let (Some(name), Some(scale)) = (operation.name, operation.scale) else {
                    results.push(McpResponse::Error(
                        "set_scale requires 'name' and 'scale' fields".to_string(),
                    ));
                    continue;
                };
                mcp_set_scale(world, name, scale)
            }
            "set_material_color" => {
                let (Some(name), Some(color)) = (operation.name, operation.color) else {
                    results.push(McpResponse::Error(
                        "set_material_color requires 'name' and 'color' fields".to_string(),
                    ));
                    continue;
                };
                mcp_set_material_color(world, name, color)
            }
            "set_emissive" => {
                let (Some(name), Some(emissive)) = (operation.name, operation.emissive) else {
                    results.push(McpResponse::Error(
                        "set_emissive requires 'name' and 'emissive' fields".to_string(),
                    ));
                    continue;
                };
                mcp_set_emissive(world, name, emissive)
            }
            "set_parent" => {
                let Some(name) = operation.name else {
                    results.push(McpResponse::Error(
                        "set_parent requires 'name' field".to_string(),
                    ));
                    continue;
                };
                mcp_set_parent(world, name, operation.parent)
            }
            "clear_scene" => mcp_clear_scene(world),
            "list_entities" => {
                let names: Vec<String> = world.resources.entity_names.keys().cloned().collect();
                McpResponse::EntityList(names)
            }
            "query_entity" => {
                let Some(name) = operation.name else {
                    results.push(McpResponse::Error(
                        "query_entity requires 'name' field".to_string(),
                    ));
                    continue;
                };
                mcp_query_entity(world, name)
            }
            "set_atmosphere" => {
                let Some(atmosphere) = operation.atmosphere else {
                    results.push(McpResponse::Error(
                        "set_atmosphere requires 'atmosphere' field".to_string(),
                    ));
                    continue;
                };
                mcp_set_atmosphere(world, atmosphere)
            }
            _ => McpResponse::Error(format!(
                "Unknown operation '{}'. Valid operations: spawn_entity, despawn_entity, set_position, set_rotation, set_scale, set_material_color, set_emissive, set_parent, query_entity, list_entities, clear_scene, set_atmosphere",
                operation.operation
            )),
        };
        results.push(response);
    }

    results
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn parse_vec3(s: &str) -> Option<[f32; 3]> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 1 {
        let v = parts[0].parse().ok()?;
        Some([v, v, v])
    } else if parts.len() == 3 {
        Some([
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ])
    } else {
        None
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn parse_vec4(s: &str) -> Option<[f32; 4]> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 4 {
        Some([
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
            parts[3].parse().ok()?,
        ])
    } else {
        None
    }
}

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
fn process_text_commands(world: &mut World, commands: &str) -> Vec<crate::mcp::McpResponse> {
    use crate::mcp::McpResponse;

    let mut results = Vec::new();

    for line in commands.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let response = match parts[0].to_lowercase().as_str() {
            "spawn" | "s" => {
                if parts.len() < 4 {
                    McpResponse::Error(
                        "spawn requires: spawn <name> <mesh> <x>,<y>,<z> [options]".to_string(),
                    )
                } else {
                    let name = parts[1].to_string();
                    let mesh = parts[2].to_string();
                    let Some(position) = parse_vec3(parts[3]) else {
                        results.push(McpResponse::Error(format!(
                            "Invalid position '{}'. Use x,y,z format",
                            parts[3]
                        )));
                        continue;
                    };

                    let mut scale = [1.0, 1.0, 1.0];
                    let mut color = None;
                    let mut emissive = None;
                    let mut parent = None;
                    let mut alpha = None;

                    for part in &parts[4..] {
                        if let Some(val) = part
                            .strip_prefix("scale:")
                            .or_else(|| part.strip_prefix("sc:"))
                            .or_else(|| part.strip_prefix("s:"))
                        {
                            if let Some(s) = parse_vec3(val) {
                                scale = s;
                            }
                        } else if let Some(val) = part
                            .strip_prefix("color:")
                            .or_else(|| part.strip_prefix("c:"))
                        {
                            color = parse_vec4(val);
                        } else if let Some(val) = part
                            .strip_prefix("emissive:")
                            .or_else(|| part.strip_prefix("e:"))
                        {
                            emissive = parse_vec3(val);
                        } else if let Some(val) = part
                            .strip_prefix("parent:")
                            .or_else(|| part.strip_prefix("p:"))
                        {
                            parent = Some(val.to_string());
                        } else if let Some(val) = part
                            .strip_prefix("alpha:")
                            .or_else(|| part.strip_prefix("a:"))
                        {
                            alpha = Some(val.to_string());
                        }
                    }

                    mcp_spawn_entity(
                        world,
                        SpawnEntityParams {
                            name,
                            mesh,
                            position,
                            scale,
                            color,
                            emissive,
                            parent_name: parent,
                            alpha_mode: alpha,
                        },
                    )
                }
            }
            "despawn" | "d" => {
                if parts.len() < 2 {
                    McpResponse::Error("despawn requires: despawn <name>".to_string())
                } else {
                    mcp_despawn_entity(world, parts[1].to_string())
                }
            }
            "pos" | "position" => {
                if parts.len() < 3 {
                    McpResponse::Error("pos requires: pos <name> <x>,<y>,<z>".to_string())
                } else {
                    let Some(position) = parse_vec3(parts[2]) else {
                        results.push(McpResponse::Error(format!(
                            "Invalid position '{}'",
                            parts[2]
                        )));
                        continue;
                    };
                    mcp_set_position(world, parts[1].to_string(), position)
                }
            }
            "rot" | "rotation" => {
                if parts.len() < 3 {
                    McpResponse::Error("rot requires: rot <name> <pitch>,<yaw>,<roll>".to_string())
                } else {
                    let Some(rotation) = parse_vec3(parts[2]) else {
                        results.push(McpResponse::Error(format!(
                            "Invalid rotation '{}'",
                            parts[2]
                        )));
                        continue;
                    };
                    mcp_set_rotation(world, parts[1].to_string(), rotation)
                }
            }
            "scale" | "sc" => {
                if parts.len() < 3 {
                    McpResponse::Error("scale requires: scale <name> <x>,<y>,<z>".to_string())
                } else {
                    let Some(scale) = parse_vec3(parts[2]) else {
                        results.push(McpResponse::Error(format!("Invalid scale '{}'", parts[2])));
                        continue;
                    };
                    mcp_set_scale(world, parts[1].to_string(), scale)
                }
            }
            "color" | "c" => {
                if parts.len() < 3 {
                    McpResponse::Error("color requires: color <name> <r>,<g>,<b>,<a>".to_string())
                } else {
                    let Some(color) = parse_vec4(parts[2]) else {
                        results.push(McpResponse::Error(format!("Invalid color '{}'", parts[2])));
                        continue;
                    };
                    mcp_set_material_color(world, parts[1].to_string(), color)
                }
            }
            "emissive" | "e" => {
                if parts.len() < 3 {
                    McpResponse::Error("emissive requires: emissive <name> <r>,<g>,<b>".to_string())
                } else {
                    let Some(emissive) = parse_vec3(parts[2]) else {
                        results.push(McpResponse::Error(format!(
                            "Invalid emissive '{}'",
                            parts[2]
                        )));
                        continue;
                    };
                    mcp_set_emissive(world, parts[1].to_string(), emissive)
                }
            }
            "parent" | "p" => {
                if parts.len() < 3 {
                    McpResponse::Error("parent requires: parent <child> <parent>".to_string())
                } else {
                    mcp_set_parent(world, parts[1].to_string(), Some(parts[2].to_string()))
                }
            }
            "unparent" => {
                if parts.len() < 2 {
                    McpResponse::Error("unparent requires: unparent <name>".to_string())
                } else {
                    mcp_set_parent(world, parts[1].to_string(), None)
                }
            }
            "clear" | "clr" => mcp_clear_scene(world),
            "list" | "ls" => {
                let names: Vec<String> = world.resources.entity_names.keys().cloned().collect();
                McpResponse::EntityList(names)
            }
            "query" | "q" => {
                if parts.len() < 2 {
                    McpResponse::Error("query requires: query <name>".to_string())
                } else {
                    mcp_query_entity(world, parts[1].to_string())
                }
            }
            "atmosphere" | "atmo" => {
                if parts.len() < 2 {
                    McpResponse::Error("atmosphere requires: atmosphere <type>".to_string())
                } else {
                    mcp_set_atmosphere(world, parts[1].to_string())
                }
            }
            "hdr" => {
                if parts.len() < 2 {
                    McpResponse::Error("hdr requires: hdr <path>".to_string())
                } else {
                    mcp_load_hdr(world, parts[1].to_string())
                }
            }
            _ => McpResponse::Error(format!(
                "Unknown command '{}'. Valid: s(pawn), d(espawn), pos, rot, sc(ale), c(olor), e(missive), p(arent), b(ehavior), clr/clear, ls/list, q(uery), atmo(sphere), hdr",
                parts[0]
            )),
        };
        results.push(response);
    }

    results
}
