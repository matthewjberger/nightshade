use crate::ecs::world::World;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub struct RenderResources {
    pub scene_color: crate::render::wgpu::rendergraph::ResourceId,
    pub depth: crate::render::wgpu::rendergraph::ResourceId,
    pub compute_output: crate::render::wgpu::rendergraph::ResourceId,
    pub swapchain: crate::render::wgpu::rendergraph::ResourceId,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_DROPPED_FILES: std::cell::RefCell<Vec<crate::ecs::input::resources::DroppedFile>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

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

    let mut context = WindowContext {
        state,
        world: World::default(),
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

pub trait State {
    fn title(&self) -> &str {
        "Nightshade"
    }
    fn icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("../images/icon.png"))
    }
    fn initialize(&mut self, _world: &mut World) {}
    fn next_state(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        None
    }
    fn configure_render_graph(
        &mut self,
        graph: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let blit_pass = crate::render::wgpu::passes::BlitPass::new(device, surface_format);

        let _ = graph.add_pass(
            Box::new(blit_pass),
            &[
                ("input", resources.scene_color),
                ("output", resources.swapchain),
            ],
        );
    }
    #[cfg(feature = "egui")]
    fn ui(&mut self, _world: &mut World, _ui_context: &egui::Context) {}
    fn immediate_ui(&mut self, _world: &mut World, _ui: &mut crate::ecs::ui::ImmediateUi) {}
    fn run_systems(&mut self, _world: &mut World) {}
    fn update_render_graph(
        &mut self,
        _graph: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        _world: &World,
    ) {
    }
    fn handle_event(&mut self, _world: &mut World, _message: &crate::ecs::world::events::Message) {}
    fn on_keyboard_input(
        &mut self,
        _world: &mut World,
        _key_code: winit::keyboard::KeyCode,
        _key_state: winit::event::ElementState,
    ) {
    }
    fn on_dropped_file(&mut self, _world: &mut World, _path: &std::path::Path) {}
    fn on_dropped_file_data(&mut self, _world: &mut World, _name: &str, _data: &[u8]) {}
    fn on_hovered_file(&mut self, _world: &mut World, _path: &std::path::Path) {}
    fn on_hovered_file_cancelled(&mut self, _world: &mut World) {}
    #[cfg(feature = "gamepad")]
    fn on_gamepad_event(&mut self, _world: &mut World, _event: gilrs::Event) {}
    fn on_mouse_input(
        &mut self,
        _world: &mut World,
        _state: winit::event::ElementState,
        _button: winit::event::MouseButton,
    ) {
    }
}

pub struct WindowContext {
    pub world: World,
    pub state: Box<dyn State>,
    pub renderer: Option<Box<dyn crate::ecs::world::Render>>,
    #[cfg(target_arch = "wasm32")]
    pub renderer_receiver:
        Option<futures::channel::oneshot::Receiver<Box<dyn crate::ecs::world::Render>>>,
}

impl winit::application::ApplicationHandler for WindowContext {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut attributes = winit::window::Window::default_attributes();

        #[cfg(not(target_arch = "wasm32"))]
        {
            attributes = attributes.with_title(self.state.title());

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

        #[cfg(not(target_arch = "wasm32"))]
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
