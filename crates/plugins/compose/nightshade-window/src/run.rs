use nightshade::prelude::*;
use nightshade_platform::prelude::*;

use nightshade::World;
use winit::event::{DeviceEvent, WindowEvent};

use crate::host::Host;
use crate::renderer::Renderer;
use crate::resources::PendingHost;

pub mod file_drop;
mod input_translation;

pub mod offscreen;
pub use offscreen::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod pump;

use crate::pacing::{
    refresh_macos_frame_rate_limit, sync_macos_auto_frame_rate_limit, throttle_to_frame_rate_limit,
};
use input_translation::forward_window_events;

#[cfg(target_arch = "wasm32")]
use crate::pacing::wasm_viewport;
#[cfg(target_arch = "wasm32")]
use file_drop::setup_wasm_file_drop;

pub fn launch<R: Renderer + 'static>(
    host: Box<dyn Host>,
    factory: RendererFactory<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    launch_with_world(host, factory, World::tracking())
}

pub fn launch_with_world<R: Renderer + 'static>(
    host: Box<dyn Host>,
    factory: RendererFactory<R>,
    world: World,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = WindowContext {
        host,
        world,
        renderer: None,
        #[cfg(target_arch = "wasm32")]
        renderer_receiver: None,
        initialized: false,
        renderer_factory: Some(factory),
        caller_driven_frame: false,
    };

    crate::platform::event_loop::run_event_loop(context)?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_min_window_size(resources: &mut nightshade::ecs::ResourceMap) {
    let desired = resources
        .res::<nightshade_platform::contract::window::Window>()
        .min_window_size;
    if resources
        .res::<nightshade_platform::contract::window::Window>()
        .applied_min_window_size
        == desired
    {
        return;
    }
    let Some(handle) = resources
        .res::<nightshade_platform::contract::window::Window>()
        .handle
        .as_ref()
    else {
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
    resources
        .res_mut::<nightshade_platform::contract::window::Window>()
        .applied_min_window_size = desired;
}

#[cfg(target_arch = "wasm32")]
fn sync_min_window_size(_resources: &mut nightshade::ecs::ResourceMap) {}

#[cfg(not(target_arch = "wasm32"))]
fn sync_window_title(resources: &mut nightshade::ecs::ResourceMap) {
    let (title, handle) = {
        let window = resources.res::<nightshade_platform::contract::window::Window>();
        if window.applied_title.as_deref() == Some(&window.title) {
            return;
        }
        (window.title.clone(), window.handle.clone())
    };
    let Some(handle) = handle else {
        return;
    };
    handle.set_title(&title);
    resources
        .res_mut::<nightshade_platform::contract::window::Window>()
        .applied_title = Some(title);
}

#[cfg(target_arch = "wasm32")]
fn sync_window_title(_resources: &mut nightshade::ecs::ResourceMap) {}

#[cfg(not(target_arch = "wasm32"))]
fn sync_window_icon(resources: &mut nightshade::ecs::ResourceMap) {
    let current = resources
        .res::<nightshade_platform::contract::window::Window>()
        .icon_bytes;
    let applied = resources
        .res::<nightshade_platform::contract::window::Window>()
        .applied_icon_bytes;
    let unchanged = match (current, applied) {
        (None, None) => true,
        (Some(left), Some(right)) => std::ptr::eq(left.as_ptr(), right.as_ptr()),
        _ => false,
    };
    if unchanged {
        return;
    }
    let Some(handle) = resources
        .res::<nightshade_platform::contract::window::Window>()
        .handle
        .as_ref()
    else {
        return;
    };
    let icon = current.and_then(|bytes| {
        let image = image::load_from_memory(bytes).ok()?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        winit::window::Icon::from_rgba(rgba.into_raw(), width, height).ok()
    });
    handle.set_window_icon(icon);
    resources
        .res_mut::<nightshade_platform::contract::window::Window>()
        .applied_icon_bytes = current;
}

#[cfg(target_arch = "wasm32")]
fn sync_window_icon(_resources: &mut nightshade::ecs::ResourceMap) {}

fn refresh_window_metrics(
    resources: &mut nightshade::ecs::ResourceMap,
    event_loop: &winit::event_loop::ActiveEventLoop,
) {
    if let Some(handle) = resources
        .res::<nightshade_platform::contract::window::Window>()
        .handle
        .as_ref()
    {
        resources
            .res_mut::<nightshade_platform::contract::window::Window>()
            .cached_scale_factor = handle.scale_factor() as f32;
    }
    crate::platform::content_rect::refresh_content_rect(
        resources.res_mut::<nightshade_platform::contract::window::Window>(),
    );
    crate::platform::monitor::refresh_monitors_from_event_loop(
        resources.res_mut::<nightshade_platform::contract::window::Monitors>(),
        event_loop,
    );
}

pub(crate) fn step<R: Renderer>(
    world: &mut World,
    host: &mut Box<dyn Host>,
    renderer: &mut Option<R>,
    event: &WindowEvent,
    caller_driven_frame: bool,
) -> Option<Box<dyn Host>> {
    sync_min_window_size(&mut world.resources);
    sync_window_title(&mut world.resources);
    sync_window_icon(&mut world.resources);
    if matches!(event, WindowEvent::RedrawRequested) && !caller_driven_frame {
        sync_macos_auto_frame_rate_limit(&mut world.resources);
        throttle_to_frame_rate_limit(&mut world.resources);
        crate::platform::content_rect::refresh_content_rect(
            world
                .resources
                .res_mut::<nightshade_platform::contract::window::Window>(),
        );
    }
    host.window_event(&mut world.storage, &mut world.resources, event);
    handle_event_systems(&mut world.resources, event, caller_driven_frame);

    if caller_driven_frame {
        return None;
    }

    match event {
        WindowEvent::RedrawRequested => match renderer.as_mut() {
            Some(renderer) => run_frame_body(world, host.as_mut(), renderer),
            None if !world
                .res::<nightshade_render::resources::viewport::Viewport>()
                .create_renderer =>
            {
                run_frame_logic(world, host.as_mut())
            }
            None => None,
        },
        event => {
            forward_window_events(&mut world.events, &mut world.resources, event);
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_surface_to_window<R: Renderer>(
    resources: &mut nightshade::ecs::ResourceMap,
    renderer: &mut R,
) {
    let (width, height) = present_size(resources);
    if width == 0 || height == 0 {
        return;
    }
    if resources
        .res::<nightshade_platform::contract::window::Window>()
        .cached_viewport_size
        == Some((width, height))
    {
        return;
    }
    resources
        .res_mut::<nightshade_platform::contract::window::Window>()
        .cached_viewport_size = Some((width, height));
    if let Err(error) = renderer.resize(width, height) {
        tracing::error!("Failed to resize the surface to the window: {error}");
    }
}

pub fn run_frame_body<R: Renderer>(
    world: &mut World,
    host: &mut dyn Host,
    renderer: &mut R,
) -> Option<Box<dyn Host>> {
    #[cfg(not(target_arch = "wasm32"))]
    sync_surface_to_window(&mut world.resources, renderer);

    if let Some(next_host) = run_frame_logic(world, host) {
        return Some(next_host);
    }

    let (viewport_width, viewport_height) = present_size(&world.resources);
    let should_render =
        (viewport_width > 0 && viewport_height > 0) || renders_off_window(&world.resources);
    if should_render {
        if let Err(error) = renderer.update(&world.storage, &mut world.resources) {
            tracing::error!("Failed to update render graph: {error}");
        }

        renderer.prepare(&mut world.storage, &mut world.resources);

        if let Err(error) = renderer.render(world) {
            tracing::error!("Failed to draw frame: {error}");
        }
    }

    None
}

pub(crate) fn run_frame_logic(world: &mut World, host: &mut dyn Host) -> Option<Box<dyn Host>> {
    world.increment_tick();

    host.run_systems(world);

    if let Some(builder) = world.res_mut::<PendingHost>().0.take() {
        let nightshade::ecs::World {
            storage, resources, ..
        } = &mut *world;
        return Some(builder(storage, resources));
    }

    None
}

fn present_size(resources: &nightshade::ecs::ResourceMap) -> (u32, u32) {
    let window = resources.res::<nightshade_platform::contract::window::Window>();
    match window.handle.as_ref() {
        Some(window_handle) => {
            let size = window_handle.inner_size();
            (size.width, size.height)
        }
        None => window.cached_viewport_size.unwrap_or((0, 0)),
    }
}

fn renders_off_window(resources: &nightshade::ecs::ResourceMap) -> bool {
    resources
        .res::<nightshade_render::contract::RenderViews>()
        .dispatched
        .iter()
        .any(|view| view.target.is_some_and(|target| target.host_owned))
}

fn handle_event_systems(
    resources: &mut nightshade::ecs::ResourceMap,
    event: &WindowEvent,
    caller_driven_frame: bool,
) {
    crate::platform::input::apply_window_event(resources, event);
    timing_system(resources, event, caller_driven_frame);
}

fn timing_system(
    resources: &mut nightshade::ecs::ResourceMap,
    event: &WindowEvent,
    caller_driven_frame: bool,
) {
    if matches!(event, WindowEvent::RedrawRequested) && !caller_driven_frame {
        advance_timing(resources);
    }
}

pub fn advance_timing(resources: &mut nightshade::ecs::ResourceMap) {
    use web_time::Instant;

    let now = Instant::now();

    let timing = resources.res_mut::<nightshade_platform::contract::time::Time>();

    let measured = timing
        .last_frame_start_instant
        .map_or(0.0, |last_frame| (now - last_frame).as_secs_f32());
    timing.raw_delta_time = timing.host_delta_time.take().unwrap_or(measured);

    timing.delta_time = if timing.paused {
        0.0
    } else {
        timing.raw_delta_time * timing.time_speed
    };

    timing.last_frame_start_instant = Some(now);

    if timing.current_frame_start_instant.is_none() {
        timing.current_frame_start_instant = Some(now);
    }

    timing.uptime_seconds += timing.raw_delta_time as f64;
    timing.uptime_milliseconds = (timing.uptime_seconds * 1000.0) as u64;

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

pub(crate) fn apply_surface_resize<R: Renderer>(
    resources: &mut nightshade::ecs::ResourceMap,
    renderer: &mut Option<R>,
    width: u32,
    height: u32,
) {
    resources
        .res_mut::<nightshade_platform::contract::window::Window>()
        .cached_viewport_size = Some((width, height));
    if let Some(renderer) = renderer.as_mut()
        && let Err(error) = renderer.resize(width, height)
    {
        tracing::error!("Failed to resize surface: {error}");
    }
}

pub type RendererFuture<R> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, Box<dyn std::error::Error>>>>>;

pub type RendererFactory<R> =
    Box<dyn FnOnce(std::sync::Arc<winit::window::Window>, u32, u32) -> RendererFuture<R>>;

pub struct WindowContext<R: Renderer> {
    pub world: World,
    pub host: Box<dyn Host>,
    pub renderer: Option<R>,
    #[cfg(target_arch = "wasm32")]
    pub renderer_receiver: Option<futures::channel::oneshot::Receiver<R>>,
    pub initialized: bool,
    pub renderer_factory: Option<RendererFactory<R>>,
    pub caller_driven_frame: bool,
}

impl<R: Renderer + 'static> WindowContext<R> {
    fn drive_hidden_frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        advance_timing(&mut self.world.resources);
        if let Some(next_host) = run_frame_body(&mut self.world, self.host.as_mut(), renderer) {
            self.host = next_host;
        }
        if self
            .world
            .res::<nightshade_platform::contract::window::Window>()
            .should_exit
        {
            event_loop.exit();
        }
    }
}

impl<R: Renderer + 'static> winit::application::ApplicationHandler for WindowContext<R> {
    #[cfg(target_os = "android")]
    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
        nightshade_platform::commands::android_input::drain_input(&mut self.world.resources);
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        nightshade_platform::commands::emit_app_event(
            &mut self.world.events,
            self.world
                .resources
                .res_mut::<nightshade_platform::contract::input::Input>(),
            nightshade_platform::contract::AppEvent::Suspended,
        );
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.release_surface();
        }
        self.world
            .res_mut::<nightshade_platform::contract::window::Window>()
            .handle = None;
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self
            .world
            .res::<nightshade_platform::contract::window::Window>()
            .handle
            .is_some()
        {
            return;
        }

        let mut attributes = winit::window::Window::default_attributes();

        #[cfg(not(target_arch = "wasm32"))]
        {
            attributes = attributes
                .with_title(
                    &self
                        .world
                        .res::<nightshade_platform::contract::window::Window>()
                        .title,
                )
                .with_resizable(true)
                .with_decorations(true)
                .with_visible(false)
                .with_min_inner_size(winit::dpi::LogicalSize::new(400.0, 300.0));

            if let Some((width, height)) = self
                .world
                .res::<nightshade_platform::contract::window::Window>()
                .initial_size
            {
                attributes =
                    attributes.with_inner_size(winit::dpi::PhysicalSize::new(width, height));
            }

            if let Some(icon_bytes) = self
                .world
                .res::<nightshade_platform::contract::window::Window>()
                .icon_bytes
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
            self.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .applied_title = Some(
                self.world
                    .res::<nightshade_platform::contract::window::Window>()
                    .title
                    .clone(),
            );
            self.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .applied_icon_bytes = self
                .world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .icon_bytes;
        }

        #[cfg(target_arch = "wasm32")]
        let (canvas_width, canvas_height, capped_dpr) = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            let viewport = wasm_viewport(800.0, 600.0);
            self.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .pointer_viewport_scale = viewport.pointer_scale();
            let (physical_width, physical_height) = viewport.physical_size();
            canvas.set_width(physical_width);
            canvas.set_height(physical_height);
            attributes = attributes.with_canvas(Some(canvas));
            (physical_width, physical_height, viewport.capped_dpr as f32)
        };

        #[cfg(target_arch = "wasm32")]
        setup_wasm_file_drop();

        let Ok(window) = event_loop.create_window(attributes) else {
            return;
        };

        let window_handle = std::sync::Arc::new(window);
        self.world
            .res_mut::<nightshade_platform::contract::window::Window>()
            .handle = Some(window_handle.clone());

        refresh_window_metrics(&mut self.world.resources, event_loop);

        #[cfg(target_arch = "wasm32")]
        {
            self.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .cached_scale_factor = capped_dpr;
        }

        sync_macos_auto_frame_rate_limit(&mut self.world.resources);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window_handle.canvas() {
                self.world
                    .res_mut::<nightshade_platform::contract::window::Window>()
                    .cached_viewport_size = Some((canvas.width(), canvas.height()));
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let dimension = window_handle.inner_size();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .cached_viewport_size = Some((dimension.width.max(1), dimension.height.max(1)));
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(renderer) = self.renderer.as_mut()
            && let Err(error) = renderer.recreate_surface(
                window_handle.clone(),
                dimension.width.max(1),
                dimension.height.max(1),
            )
        {
            tracing::error!("Failed to rebuild the surface on resume: {error}");
        }

        #[cfg(not(target_arch = "wasm32"))]
        if self.renderer.is_none()
            && self
                .world
                .res::<nightshade_render::resources::viewport::Viewport>()
                .create_renderer
            && let Some(factory) = self.renderer_factory.take()
        {
            let width = dimension.width.max(1);
            let height = dimension.height.max(1);
            match pollster::block_on(factory(window_handle.clone(), width, height)) {
                Ok(mut renderer) => {
                    if let Err(error) = renderer.configure() {
                        panic!("the renderer could not configure the surface: {error}");
                    }
                    self.renderer = Some(renderer);
                }
                Err(error) => {
                    panic!("the renderer could not open on this device: {error}");
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(renderer) = self.renderer.as_mut()
            && let Err(error) =
                renderer.recreate_surface(window_handle.clone(), canvas_width, canvas_height)
        {
            tracing::error!("Failed to rebuild the surface on resume: {error}");
        }

        #[cfg(target_arch = "wasm32")]
        if self.renderer.is_none()
            && self
                .world
                .res::<nightshade_render::resources::viewport::Viewport>()
                .create_renderer
            && let Some(factory) = self.renderer_factory.take()
        {
            let (sender, receiver) = futures::channel::oneshot::channel();
            self.renderer_receiver = Some(receiver);

            let window_handle_clone = std::sync::Arc::clone(&window_handle);
            let built = factory(window_handle_clone, canvas_width, canvas_height);

            wasm_bindgen_futures::spawn_local(async move {
                match built.await {
                    Ok(renderer) => {
                        if sender.send(renderer).is_err() {
                            panic!("the window stopped waiting for the renderer");
                        }
                    }
                    Err(error) => {
                        panic!("the renderer could not open on this device: {error}");
                    }
                }
            });
        }

        #[cfg(target_arch = "wasm32")]
        if !self
            .world
            .res::<nightshade_render::resources::viewport::Viewport>()
            .create_renderer
            && !self.initialized
        {
            self.host.initialize(&mut self.world);
            sync_min_window_size(&mut self.world.resources);
            nightshade::ecs::system_param::run_system(
                &mut self.world,
                crate::systems::canvas::sync_wasm_canvas_size,
            );
            self.initialized = true;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if !self.initialized {
                self.host.initialize(&mut self.world);
                sync_min_window_size(&mut self.world.resources);
                #[cfg(target_arch = "wasm32")]
                nightshade::ecs::system_param::run_system(
                    &mut self.world,
                    crate::systems::canvas::sync_wasm_canvas_size,
                );
                self.initialized = true;
            }

            nightshade_platform::commands::emit_app_event(
                &mut self.world.events,
                self.world
                    .resources
                    .res_mut::<nightshade_platform::contract::input::Input>(),
                nightshade_platform::contract::AppEvent::Resumed,
            );

            let window_handle = self
                .world
                .res::<nightshade_platform::contract::window::Window>()
                .handle
                .clone();
            if self
                .world
                .res::<nightshade_platform::contract::window::Window>()
                .use_fullscreen
                && let Some(window_handle) = &window_handle
            {
                window_handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }

            if let Some(window_handle) = &window_handle {
                let show_cursor = self
                    .world
                    .res::<nightshade_platform::contract::window::Window>()
                    .show_cursor;
                window_handle.set_cursor_visible(show_cursor);
                if !self
                    .world
                    .res::<nightshade_platform::contract::window::Window>()
                    .start_hidden
                {
                    window_handle.set_visible(true);
                }
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
                if let Err(error) = renderer.configure() {
                    panic!("the renderer could not configure the surface: {error}");
                }
                self.renderer = Some(renderer);
                self.host.initialize(&mut self.world);
                sync_min_window_size(&mut self.world.resources);
                #[cfg(target_arch = "wasm32")]
                nightshade::ecs::system_param::run_system(
                    &mut self.world,
                    crate::systems::canvas::sync_wasm_canvas_size,
                );

                let show_cursor = self
                    .world
                    .res::<nightshade_platform::contract::window::Window>()
                    .show_cursor;
                if let Some(window_handle) = &self
                    .world
                    .res::<nightshade_platform::contract::window::Window>()
                    .handle
                {
                    window_handle.set_cursor_visible(show_cursor);
                }

                renderer_received = true;
            }
            if renderer_received {
                self.renderer_receiver = None;
            }
        }

        if matches!(event, winit::event::WindowEvent::CloseRequested) {
            self.world
                .res_mut::<nightshade_platform::contract::window::Window>()
                .close_requested = true;
        }

        match event {
            winit::event::WindowEvent::Focused(focused) => {
                self.world
                    .res_mut::<nightshade_platform::contract::window::Window>()
                    .is_focused = focused;
                if !focused {
                    self.world
                        .res_mut::<nightshade_platform::contract::input::Input>()
                        .mouse
                        .position_initialized = false;
                }
            }
            winit::event::WindowEvent::Moved(_) => {
                if let Some(handle) = self
                    .world
                    .res::<nightshade_platform::contract::window::Window>()
                    .handle
                    .clone()
                {
                    refresh_macos_frame_rate_limit(&mut self.world.resources, &handle);
                }
            }
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                refresh_window_metrics(&mut self.world.resources, event_loop);
                #[cfg(target_arch = "wasm32")]
                {
                    let viewport = wasm_viewport(0.0, 0.0);
                    let window = self
                        .world
                        .res_mut::<nightshade_platform::contract::window::Window>();
                    window.pointer_viewport_scale = viewport.pointer_scale();
                    window.cached_scale_factor = viewport.capped_dpr as f32;
                }
                if let Some(handle) = self
                    .world
                    .res::<nightshade_platform::contract::window::Window>()
                    .handle
                    .clone()
                {
                    refresh_macos_frame_rate_limit(&mut self.world.resources, &handle);
                }
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height })
                if width > 0 && height > 0 =>
            {
                refresh_window_metrics(&mut self.world.resources, event_loop);

                #[cfg(target_arch = "wasm32")]
                let (surface_width, surface_height) = {
                    let viewport = wasm_viewport(width as f64, height as f64);
                    {
                        let window = self
                            .world
                            .res_mut::<nightshade_platform::contract::window::Window>();
                        window.pointer_viewport_scale = viewport.pointer_scale();
                        window.cached_scale_factor = viewport.capped_dpr as f32;
                    }
                    let (physical_width, physical_height) = viewport.physical_size();
                    use winit::platform::web::WindowExtWebSys;
                    if let Some(window) = self
                        .world
                        .res::<nightshade_platform::contract::window::Window>()
                        .handle
                        .as_ref()
                        && let Some(canvas) = window.canvas()
                    {
                        canvas.set_width(physical_width);
                        canvas.set_height(physical_height);
                    }
                    (physical_width, physical_height)
                };
                #[cfg(not(target_arch = "wasm32"))]
                let (surface_width, surface_height) = (width, height);

                apply_surface_resize(
                    &mut self.world.resources,
                    &mut self.renderer,
                    surface_width,
                    surface_height,
                );
            }
            _ => {}
        }

        if let Some(new_host) = crate::run::step(
            &mut self.world,
            &mut self.host,
            &mut self.renderer,
            &event,
            self.caller_driven_frame,
        ) {
            self.host = new_host;
        }

        let window = self
            .world
            .res_mut::<nightshade_platform::contract::window::Window>();
        if matches!(event, winit::event::WindowEvent::RedrawRequested) && window.close_requested {
            window.should_exit = true;
        }
        if window.should_exit {
            event_loop.exit();
            return;
        }

        run_system(&mut self.world, request_redraw_system);
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let mouse = &mut self
                .world
                .res_mut::<nightshade_platform::contract::input::Input>()
                .mouse;
            mouse.raw_mouse_delta.x += delta.0 as f32;
            mouse.raw_mouse_delta.y += delta.1 as f32;
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self
            .world
            .res::<nightshade_platform::contract::window::Window>()
            .start_hidden
        {
            self.drive_hidden_frame(event_loop);
            return;
        }
        run_system(&mut self.world, request_redraw_system);
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
}

pub(crate) fn request_redraw_system(
    mut window: nightshade::ecs::system_param::ResMut<
        nightshade_platform::contract::window::Window,
    >,
) {
    let Some(window_handle) = window.handle.as_mut() else {
        return;
    };
    let winit::dpi::PhysicalSize { width, height } = window_handle.inner_size();
    if width == 0 || height == 0 {
        return;
    }
    window_handle.request_redraw();
}
