use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowId};

use crate::platform::input::apply_window_event;
use nightshade_platform::KeyCode;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::window::Window;
use nightshade_platform::queries::input::keyboard_for_active;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("nightshade"),
            width: 1280,
            height: 720,
        }
    }
}

pub struct InitContext {
    pub window: Arc<WinitWindow>,
    pub width: u32,
    pub height: u32,
}

pub struct FrameContext {
    pub width: u32,
    pub height: u32,
    pub delta_time: f32,
    pub elapsed: f32,
}

pub trait WindowApp: Sized {
    fn init(
        storage: &mut nightshade::ecs::Storage,
        resources: &mut nightshade::ecs::ResourceMap,
        context: InitContext,
    ) -> Self;

    fn frame(&mut self, ecs: &mut nightshade::ecs::World, context: FrameContext);

    fn resize(&mut self, _width: u32, _height: u32) {}
}

pub fn run<A>(config: WindowConfig) -> Result<(), winit::error::EventLoopError>
where
    A: WindowApp + 'static,
{
    let mut ecs = nightshade::ecs::World::tracking();
    ecs.insert_resource(Input::default());
    ecs.insert_resource(Window::default());

    crate::platform::event_loop::run_event_loop(Runner::<A> {
        config,
        ecs,
        window: None,
        app: None,
        size: (0, 0),
        last_frame: None,
        start: None,
    })
}

struct Runner<A: WindowApp> {
    config: WindowConfig,
    ecs: nightshade::ecs::World,
    window: Option<Arc<WinitWindow>>,
    app: Option<A>,
    size: (u32, u32),
    last_frame: Option<Instant>,
    start: Option<Instant>,
}

impl<A: WindowApp> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.is_some() {
            return;
        }

        let attributes = WinitWindow::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(PhysicalSize::new(self.config.width, self.config.height));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        self.ecs.res_mut::<Window>().handle = Some(window.clone());
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        self.size = (width, height);

        let app = A::init(
            &mut self.ecs.storage,
            &mut self.ecs.resources,
            InitContext {
                window: window.clone(),
                width,
                height,
            },
        );

        let now = Instant::now();
        self.start = Some(now);
        self.last_frame = Some(now);
        self.window = Some(window.clone());
        self.app = Some(app);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        apply_window_event(&mut self.ecs.resources, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { .. } => {
                if keyboard_for_active(&self.ecs.resources).is_key_pressed(KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(new_size) => {
                let width = new_size.width.max(1);
                let height = new_size.height.max(1);
                self.size = (width, height);
                if let Some(app) = self.app.as_mut() {
                    app.resize(width, height);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = self
                    .last_frame
                    .map(|previous| (now - previous).as_secs_f32())
                    .unwrap_or(0.0);
                let elapsed = self
                    .start
                    .map(|start| (now - start).as_secs_f32())
                    .unwrap_or(0.0);
                self.last_frame = Some(now);

                let (width, height) = self.size;
                if let Some(app) = self.app.as_mut() {
                    app.frame(
                        &mut self.ecs,
                        FrameContext {
                            width,
                            height,
                            delta_time,
                            elapsed,
                        },
                    );
                }

                let input = self.ecs.res_mut::<Input>();
                input.keyboard.end_frame();
                input.mouse.end_frame();
                input.touch.end_frame();
                input.events.clear();

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
