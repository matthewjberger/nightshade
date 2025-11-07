use crate::ecs::world::{Resources, World, resources::*};
use web_time::Instant;
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

pub fn launch(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        console_log::init().expect("Failed to initialize logger");
    }

    let mut context = WindowContext {
        state: Box::new(state),
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
) {
    match event {
        WindowEvent::RedrawRequested => {
            update_frame_timing_system(world);
            crate::ecs::input::systems::gamepad_input_system(world);
            crate::ecs::transform::systems::update_global_transforms_system(world);
            create_ui_system(world, state);
            state.run_systems(world);
            crate::ecs::event_bus::systems::process_events_system(world, |world, message| {
                state.handle_event(world, message);
            });
            crate::ecs::transform::systems::update_global_transforms_system(world);
            crate::ecs::world::process_commands_system(world);
            crate::ecs::input::systems::reset_mouse_system(world);

            let Some(window_handle) = world.resources.window.handle.as_ref() else {
                return;
            };

            let Some(renderer) = renderer.as_mut() else {
                return;
            };

            let ui_frame_output = world.resources.user_interface.frame_output.take();

            let should_render =
                window_handle.inner_size().width > 0 && window_handle.inner_size().height > 0;
            if should_render {
                let (ui_output, ui_primitives) = match ui_frame_output {
                    Some((output, primitives)) => (Some(output), Some(primitives)),
                    None => (None, None),
                };
                if let Err(error) = renderer.render_frame(world, ui_output, ui_primitives) {
                    log::error!("Failed to draw frame: {error}");
                }
            }
        }
        event => {
            crate::ecs::input::systems::receive_keyboard_input(world, event);
            crate::ecs::input::systems::receive_mouse_input(world, event);
            receive_ui_event(world, event);
            forward_window_events(world, state, event);
        }
    }
}

fn forward_window_events(
    world: &mut World,
    state: &mut Box<dyn State + 'static>,
    event: &WindowEvent,
) {
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
}

pub trait State {
    fn title(&self) -> &str {
        "Nightshade"
    }
    fn initialize(&mut self, _world: &mut World) {}
    fn ui(&mut self, _world: &mut World, _ui_context: &egui::Context) {}
    fn run_systems(&mut self, _world: &mut World) {}
    fn handle_event(&mut self, _world: &mut World, _message: &crate::ecs::world::events::Message) {}
    fn on_keyboard_input(
        &mut self,
        _world: &mut World,
        _key_code: winit::keyboard::KeyCode,
        _key_state: winit::event::ElementState,
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

        let Ok(window) = event_loop.create_window(attributes) else {
            return;
        };

        let window_handle = std::sync::Arc::new(window);
        self.world.resources.window.handle = Some(window_handle.clone());

        #[cfg(not(target_arch = "wasm32"))]
        let dimension = window_handle.inner_size();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            match runtime.block_on(crate::render::core::create_renderer(
                window_handle.clone(),
                dimension.width,
                dimension.height,
            )) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                }
                Err(error) => {
                    log::error!("Failed to create renderer: {error}");
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
                            log::error!("Failed to send renderer");
                        }
                    }
                    Err(error) => {
                        log::error!("Failed to create renderer: {error}");
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            initialize_user_interface(&mut self.world);
            self.state.initialize(&mut self.world);
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
            if let Some(receiver) = self.renderer_receiver.as_mut() {
                if let Ok(Some(renderer)) = receiver.try_recv() {
                    self.renderer = Some(renderer);
                    initialize_user_interface(&mut self.world);
                    self.state.initialize(&mut self.world);
                    renderer_received = true;
                }
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

        if let winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) =
            event
            && width > 0
            && height > 0
        {
            if let Some(renderer) = &mut self.renderer
                && let Err(error) = renderer.resize_surface(width, height)
            {
                log::error!("Failed to resize surface: {error}");
            }
            sync_ui_scale_factor(&mut self.world);
        }

        crate::run::step(&mut self.world, &mut self.state, &mut self.renderer, &event);

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

fn initialize_user_interface(world: &mut World) {
    let window_handle = {
        let Some(window_handle) = world.resources.window.handle.as_mut() else {
            return;
        };
        window_handle.clone()
    };

    let gui_context = egui::Context::default();

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

fn receive_ui_event(world: &mut World, event: &WindowEvent) {
    let Some(gui_state) = &mut world.resources.user_interface.state else {
        return;
    };
    let Some(window_handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    world.resources.user_interface.consumed_event =
        gui_state.on_window_event(window_handle, event).consumed;
}

fn sync_ui_scale_factor(world: &mut World) {
    let Some(window_handle) = world.resources.window.handle.as_ref() else {
        return;
    };
    let Some(gui_state) = world.resources.user_interface.state.as_mut() else {
        return;
    };
    gui_state
        .egui_ctx()
        .set_pixels_per_point(window_handle.scale_factor() as f32);
}

fn create_ui_system(world: &mut World, state: &mut Box<dyn State + 'static>) {
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
        let gui_input = gui_state.take_egui_input(window_handle);
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

fn update_frame_timing_system(world: &mut World) {
    let now = Instant::now();

    let World {
        resources:
            Resources {
                window:
                    Window {
                        timing:
                            WindowTiming {
                                delta_time,
                                raw_delta_time,
                                time_speed,
                                last_frame_start_instant,
                                current_frame_start_instant,
                                initial_frame_start_instant,
                                frame_counter,
                                uptime_milliseconds,
                                frames_per_second,
                                ..
                            },
                        ..
                    },
                ..
            },
        ..
    } = world;

    if initial_frame_start_instant.is_none() {
        *initial_frame_start_instant = Some(now);
    }

    *raw_delta_time =
        last_frame_start_instant.map_or(0.0, |last_frame| (now - last_frame).as_secs_f32());

    *delta_time = *raw_delta_time * *time_speed;

    *last_frame_start_instant = Some(now);

    if current_frame_start_instant.is_none() {
        *current_frame_start_instant = Some(now);
    }

    if let Some(app_start) = initial_frame_start_instant.as_ref() {
        *uptime_milliseconds = (now - *app_start).as_millis() as u64;
    }

    *frame_counter += 1;
    if let Some(start) = current_frame_start_instant.as_ref() {
        if (now - *start).as_secs_f32() >= 1.0 {
            *frames_per_second = *frame_counter as f32;
            *frame_counter = 0;
            *current_frame_start_instant = Some(now);
        }
    } else {
        *current_frame_start_instant = Some(now);
    }
}
