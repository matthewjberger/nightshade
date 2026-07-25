//! Application entry point and main loop.
//!
//! Programs normally compose an [`App`](crate::app::App) from plugins and
//! call its `run`, which drives this loop. [`State`] and [`launch`] are the
//! low-level escape hatch for custom runners that own their own state type:
//!
//! ```ignore
//! struct MyGame;
//!
//! impl State for MyGame {
//!     fn initialize(&mut self, world: &mut World) {
//!         world.res_mut::<nightshade::platform::window::Window>().title = "My Game".to_string();
//!         // Set up camera, lights, entities
//!     }
//!
//!     fn run_systems(&mut self, world: &mut World) {
//!         // Per-frame game logic
//!     }
//! }
//!
//! fn main() {
//!     nightshade::run::launch(MyGame).unwrap();
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
//! | `run_systems()` | Every frame | Game logic, drain `world.res::<crate::platform::input::resources::Input>().events` |
//! | `configure_render_graph()` | Once at startup | Custom post-processing |
//! | `update_render_graph()` | Every frame | Dynamic render graph changes |
//! | `pre_render()` | Each frame before render | Custom GPU uploads |
//!
//! Input, file-drop, gamepad, and lifecycle events arrive on
//! `world.res::<crate::platform::input::resources::Input>().events` as [`AppEvent`](crate::platform::input::events::AppEvent)
//! values. Drain them in [`State::run_systems`].
//!
//! To transition to a different `State`, set
//! `world.res::<PendingState>().0 = Some(Box::new(|world| Box::new(NewState::new(world))))`
//! from inside `run_systems`; the engine takes the builder at the end of the
//! frame and invokes it with `&mut World` so the new state can be constructed
//! against the live world.
//!
//! # Typical Initialize Pattern
//!
//! ```ignore
//! fn initialize(&mut self, world: &mut World) {
//!     // Configure graphics
//!     world.res::<crate::render::config::RenderSettings>().atmosphere = Atmosphere::Sky;
//!     world.res::<crate::render::config::DebugDraw>().show_grid = true;
//!
//!     // Spawn camera
//!     let camera = spawn_camera(world, Vec3::new(5.0, 3.0, 5.0), "Camera".to_string());
//!     world.res::<crate::ecs::camera::resources::ActiveCamera>().0 = Some(camera);
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
//! closure to the [`PendingState`](crate::state::PendingState) resource from
//! inside [`State::run_systems`]. The closure receives `&mut World` so the new
//! state can be constructed against live engine data.
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     if self.should_go_to_menu {
//!         world.res::<PendingState>().0 =
//!             Some(Box::new(|_world| Box::new(MenuState::default())));
//!     }
//! }
//! ```
//!
//! # Headless and Offscreen Rendering
//!
//! [`launch`] owns a winit event loop. To run the same engine without a window,
//! against an `OffscreenCanvas` in a web worker or a native headless target,
//! drive it through the [`offscreen`] module: [`initialize_offscreen`],
//! [`tick_offscreen`], and [`resize_offscreen`] share this loop's frame body, so
//! there is one render path rather than two. See the `nightshade-worker-leptos`
//! demo for a complete web worker port.

use crate::prelude::*;

use crate::ecs::world::World;
#[cfg(target_arch = "wasm32")]
use crate::render::wgpu;
use winit::event::{DeviceEvent, WindowEvent};

pub use crate::state::{NextStateBuilder, RenderResources, State};

mod file_drop;
mod input_translation;
mod pacing;

pub mod offscreen;
pub use offscreen::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod pump;

use file_drop::process_dropped_files;
use input_translation::{forward_window_events, input_event_system};
use pacing::{
    refresh_macos_frame_rate_limit, sync_macos_auto_frame_rate_limit, throttle_to_frame_rate_limit,
};

#[cfg(target_arch = "wasm32")]
use file_drop::setup_wasm_file_drop;
#[cfg(not(target_arch = "wasm32"))]
use nightshade_platform::input::translation::recenter_locked_cursor;
#[cfg(target_arch = "wasm32")]
use pacing::wasm_capped_dpr;

/// Launches the application with the given state.
pub fn launch(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    launch_windowed(state)
}

/// Launches a windowed application (desktop or web).
pub fn launch_windowed(state: impl State + 'static) -> Result<(), Box<dyn std::error::Error>> {
    launch_with_world(state, World::default())
}

/// Launches the event loop over a world the caller already built, which is
/// how a plugin-composed [`App`](crate::app::App) carries its registered
/// member worlds, resources, and settings into the runner.
pub fn launch_with_world(
    state: impl State + 'static,
    world: World,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Box::new(state);

    #[cfg(not(target_arch = "wasm32"))]
    let context = WindowContext {
        state,
        world,
        renderer: None,
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            feature = "core"
        ))]
        accesskit: None,
        initialized: false,
        caller_driven_frame: false,
    };

    #[cfg(target_arch = "wasm32")]
    let context = WindowContext {
        state,
        world,
        renderer: None,
        renderer_receiver: None,
        initialized: false,
        caller_driven_frame: false,
    };

    nightshade_platform::run::run_event_loop(context)?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_min_window_size(world: &mut World) {
    let desired = world
        .res::<crate::platform::window::Window>()
        .min_window_size;
    if world
        .res::<crate::platform::window::Window>()
        .applied_min_window_size
        == desired
    {
        return;
    }
    let Some(handle) = world
        .res::<crate::platform::window::Window>()
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
    world
        .res_mut::<crate::platform::window::Window>()
        .applied_min_window_size = desired;
}

#[cfg(target_arch = "wasm32")]
fn sync_min_window_size(_world: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
fn sync_window_title(world: &mut World) {
    let (title, handle) = {
        let window = world.res::<crate::platform::window::Window>();
        if window.applied_title.as_deref() == Some(&window.title) {
            return;
        }
        (window.title.clone(), window.handle.clone())
    };
    let Some(handle) = handle else {
        return;
    };
    handle.set_title(&title);
    world
        .res_mut::<crate::platform::window::Window>()
        .applied_title = Some(title);
}

#[cfg(target_arch = "wasm32")]
fn sync_window_title(_world: &mut World) {}

#[cfg(all(not(target_arch = "wasm32"), feature = "assets"))]
fn sync_window_icon(world: &mut World) {
    let current = world.res::<crate::platform::window::Window>().icon_bytes;
    let applied = world
        .res::<crate::platform::window::Window>()
        .applied_icon_bytes;
    let unchanged = match (current, applied) {
        (None, None) => true,
        (Some(left), Some(right)) => std::ptr::eq(left.as_ptr(), right.as_ptr()),
        _ => false,
    };
    if unchanged {
        return;
    }
    let Some(handle) = world
        .res::<crate::platform::window::Window>()
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
    world
        .res_mut::<crate::platform::window::Window>()
        .applied_icon_bytes = current;
}

#[cfg(any(target_arch = "wasm32", not(feature = "assets")))]
fn sync_window_icon(_world: &mut World) {}

pub(crate) fn step(
    world: &mut World,
    state: &mut Box<dyn State + 'static>,
    renderer: &mut Option<crate::render::wgpu::WgpuRenderer>,
    event: &WindowEvent,
    caller_driven_frame: bool,
) -> Option<Box<dyn State>> {
    let delta_ms = (world.res::<crate::ecs::time::Time>().delta_time * 1000.0) as u32;
    let _span = tracing::info_span!("frame", delta_ms).entered();

    sync_min_window_size(world);
    sync_window_title(world);
    sync_window_icon(world);
    if matches!(event, WindowEvent::RedrawRequested) && !caller_driven_frame {
        sync_macos_auto_frame_rate_limit(world);
        throttle_to_frame_rate_limit(world);
    }
    handle_event_systems(world, event);

    // A caller-owned loop runs the frame itself, on its own schedule. The window
    // still asks for redraws through `about_to_wait`, and answering them here
    // too would run every system and render every camera a second time per
    // iteration.
    if caller_driven_frame {
        return None;
    }

    match event {
        WindowEvent::RedrawRequested => match renderer.as_mut() {
            Some(renderer) => run_frame_body(world, state.as_mut(), renderer),
            None if !world.res::<crate::viewport::Viewport>().create_renderer => {
                run_frame_logic(world, state.as_mut())
            }
            None => None,
        },
        event => {
            forward_window_events(world, event);
            None
        }
    }
}

/// Runs a single rendered frame: the user [`State::run_systems`] (which
/// runs the per-frame stages, engine systems included) and the render graph.
///
/// This is the shared body behind both the winit event loop ([`step`]) and the
/// offscreen driver ([`tick_offscreen`]). It renders whenever a viewport size is
/// known, with or without a winit window handle, so it drives an
/// `OffscreenCanvas` inside a web worker exactly as it drives a real window.
///
/// Returns the next [`State`] when [`State::run_systems`] requested a transition
/// through `world.res::<crate::state::PendingState>()`.
pub fn run_frame_body(
    world: &mut World,
    state: &mut dyn State,
    renderer: &mut crate::render::wgpu::WgpuRenderer,
) -> Option<Box<dyn State>> {
    if let Some(next_state) = run_frame_logic(world, state) {
        return Some(next_state);
    }

    let (viewport_width, viewport_height) = if let Some(window_handle) = world
        .res::<crate::platform::window::Window>()
        .handle
        .as_ref()
    {
        let size = window_handle.inner_size();
        (size.width, size.height)
    } else {
        world
            .res::<crate::platform::window::Window>()
            .cached_viewport_size
            .unwrap_or((0, 0))
    };
    // A zero-sized window means nothing to present to, which normally means
    // nothing to render. A camera drawing into a texture the host owns is the
    // exception: its frame is going somewhere else and still has a deadline, so
    // the window being minimized cannot be allowed to stop it.
    let renders_off_window = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RENDER_TARGET)
        .any(|entity| {
            world
                .get::<crate::ecs::camera::components::RenderTarget>(entity)
                .is_some_and(|target| target.is_host_owned())
        });
    let should_render = (viewport_width > 0 && viewport_height > 0) || renders_off_window;
    if should_render {
        if let Err(error) = crate::render_driver::update_with_state(renderer, state, world) {
            tracing::error!("Failed to update render graph: {error}");
        }

        state.pre_render(renderer, world);

        if let Err(error) = crate::render_driver::render_frame(renderer, world) {
            tracing::error!("Failed to draw frame: {error}");
        }
    }

    None
}

/// Runs the renderer-free portion of a frame: the user
/// [`State::run_systems`], which drives the per-frame stages (engine systems
/// included). This is the whole frame for app compositions that omit the
/// render plugin.
pub(crate) fn run_frame_logic(world: &mut World, state: &mut dyn State) -> Option<Box<dyn State>> {
    world.ecs.worlds[CORE].increment_tick();

    process_dropped_files(world);

    #[cfg(target_arch = "wasm32")]
    crate::ecs::world::commands::sync_cursor_lock_state(world);

    #[cfg(not(target_arch = "wasm32"))]
    recenter_locked_cursor(&mut world.ecs);
    #[cfg(feature = "egui")]
    crate::egui::begin_frame(world);
    state.run_systems(world);
    world
        .res_mut::<crate::platform::input::resources::Input>()
        .events
        .clear();

    #[cfg(feature = "egui")]
    crate::egui::finish_frame(world);

    if let Some(builder) = world.res_mut::<crate::state::PendingState>().0.take() {
        return Some(builder(world));
    }

    let window_handle = world
        .res::<crate::platform::window::Window>()
        .handle
        .clone();
    if world
        .res::<crate::ui::resources::RetainedUiRuntime>()
        .visible
        && let Some(window_handle) = window_handle
    {
        let requested = world
            .res::<crate::ui::resources::RetainedUiInteraction>()
            .requested_cursor;
        let applied = world
            .res::<crate::ui::resources::RetainedUiInteraction>()
            .applied_cursor;
        if requested.is_some() && requested != applied {
            if let Some(cursor) = requested {
                window_handle.set_cursor(cursor);
            }
            world
                .res_mut::<crate::ui::resources::RetainedUiInteraction>()
                .applied_cursor = requested;
        } else if requested.is_none()
            && applied.is_some()
            && applied != Some(winit::window::CursorIcon::Default)
        {
            window_handle.set_cursor(winit::window::CursorIcon::Default);
            world
                .res_mut::<crate::ui::resources::RetainedUiInteraction>()
                .applied_cursor = Some(winit::window::CursorIcon::Default);
        }
    }

    None
}

fn handle_event_systems(world: &mut World, event: &WindowEvent) {
    let _span = tracing::info_span!("event_handling").entered();
    #[cfg(feature = "egui")]
    crate::egui::handle_window_event(world, event);
    input_event_system(world, event);
    timing_system(world, event);
}

fn timing_system(world: &mut World, event: &WindowEvent) {
    let _span = tracing::info_span!("timing").entered();
    if let WindowEvent::RedrawRequested = event {
        advance_timing(world);
    }
}

/// Advances frame timing by one frame: `delta_time`, the fps averaging window,
/// and `uptime_milliseconds` on the `world.res::<crate::ecs::time::Time>()` resource.
///
/// The winit loop calls this on every `RedrawRequested`. Offscreen and headless
/// callers invoke it once per frame through [`tick_offscreen`].
pub fn advance_timing(world: &mut World) {
    use web_time::Instant;

    let now = Instant::now();

    let timing = world.res_mut::<crate::ecs::time::Time>();

    if timing.initial_frame_start_instant.is_none() {
        timing.initial_frame_start_instant = Some(now);
    }

    timing.raw_delta_time = timing
        .last_frame_start_instant
        .map_or(0.0, |last_frame| (now - last_frame).as_secs_f32());

    timing.delta_time = if timing.paused {
        0.0
    } else {
        timing.raw_delta_time * timing.time_speed
    };

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

pub(crate) fn apply_surface_resize(
    world: &mut World,
    renderer: &mut Option<crate::render::wgpu::WgpuRenderer>,
    width: u32,
    height: u32,
) {
    world
        .res_mut::<crate::platform::window::Window>()
        .cached_viewport_size = Some((width, height));
    if let Some(renderer) = renderer.as_mut()
        && let Err(error) = renderer.resize_surface(width, height)
    {
        tracing::error!("Failed to resize surface: {error}");
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
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "core"
    ))]
    pub accesskit: Option<crate::ui::accessibility::AccessKitHost>,
    pub initialized: bool,
    /// Set by a caller-owned loop that runs the frame itself. Redraw requests
    /// from the window are then answered by that loop rather than here, so a
    /// frame is not run twice per iteration.
    pub caller_driven_frame: bool,
}

impl winit::application::ApplicationHandler for WindowContext {
    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        crate::ecs::events::emit_app_event(
            &mut self.world,
            crate::platform::input::events::AppEvent::Suspended,
        );
        self.renderer = None;
        self.world
            .res_mut::<crate::platform::window::Window>()
            .handle = None;
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            feature = "core"
        ))]
        {
            self.accesskit = None;
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self
            .world
            .res::<crate::platform::window::Window>()
            .handle
            .is_some()
        {
            return;
        }

        let mut attributes = winit::window::Window::default_attributes();

        #[cfg(not(target_arch = "wasm32"))]
        {
            attributes = attributes
                .with_title(&self.world.res::<crate::platform::window::Window>().title)
                .with_resizable(true)
                .with_decorations(true)
                .with_visible(false)
                .with_min_inner_size(winit::dpi::LogicalSize::new(400.0, 300.0));

            if let Some((width, height)) = self
                .world
                .res::<crate::platform::window::Window>()
                .initial_size
            {
                attributes =
                    attributes.with_inner_size(winit::dpi::PhysicalSize::new(width, height));
            }

            #[cfg(feature = "assets")]
            if let Some(icon_bytes) = self
                .world
                .res::<crate::platform::window::Window>()
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
                .res_mut::<crate::platform::window::Window>()
                .applied_title = Some(
                self.world
                    .res::<crate::platform::window::Window>()
                    .title
                    .clone(),
            );
            self.world
                .res_mut::<crate::platform::window::Window>()
                .applied_icon_bytes = self
                .world
                .res_mut::<crate::platform::window::Window>()
                .icon_bytes;
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
            self.world
                .res_mut::<crate::platform::window::Window>()
                .pointer_viewport_scale = (dpr / raw_dpr) as f32;
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
        self.world
            .res_mut::<crate::platform::window::Window>()
            .cached_scale_factor = window_handle.scale_factor() as f32;
        self.world
            .res_mut::<crate::platform::window::Window>()
            .handle = Some(window_handle.clone());

        #[cfg(feature = "egui")]
        crate::egui::initialize(&mut self.world);

        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            feature = "core"
        ))]
        {
            self.accesskit = Some(crate::ui::accessibility::AccessKitHost::new(
                event_loop,
                &window_handle,
            ));
        }

        crate::platform::window::refresh_monitors_from_event_loop(
            self.world.res_mut::<crate::platform::window::Monitors>(),
            event_loop,
        );

        sync_macos_auto_frame_rate_limit(&mut self.world);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window_handle.canvas() {
                self.world
                    .res_mut::<crate::platform::window::Window>()
                    .cached_viewport_size = Some((canvas.width(), canvas.height()));
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let dimension = window_handle.inner_size();

        #[cfg(not(target_arch = "wasm32"))]
        if self
            .world
            .res::<crate::viewport::Viewport>()
            .create_renderer
        {
            match pollster::block_on(crate::render::core::create_renderer(
                window_handle.clone(),
                dimension.width,
                dimension.height,
            )) {
                Ok(mut renderer) => {
                    if let Err(error) = crate::render_driver::configure_with_state(
                        &mut renderer,
                        self.state.as_mut(),
                    ) {
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
        if self
            .world
            .res::<crate::viewport::Viewport>()
            .create_renderer
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

        #[cfg(target_arch = "wasm32")]
        if !self
            .world
            .res::<crate::viewport::Viewport>()
            .create_renderer
            && !self.initialized
        {
            self.state.initialize(&mut self.world);
            sync_min_window_size(&mut self.world);
            crate::ecs::camera::systems::sync_wasm_canvas_size(&mut self.world);
            self.initialized = true;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if !self.initialized {
                self.state.initialize(&mut self.world);
                sync_min_window_size(&mut self.world);
                #[cfg(target_arch = "wasm32")]
                crate::ecs::camera::systems::sync_wasm_canvas_size(&mut self.world);
                self.initialized = true;
            }

            crate::ecs::events::emit_app_event(
                &mut self.world,
                crate::platform::input::events::AppEvent::Resumed,
            );

            let window_handle = self
                .world
                .res::<crate::platform::window::Window>()
                .handle
                .clone();
            if self
                .world
                .res::<crate::platform::window::Window>()
                .use_fullscreen
                && let Some(window_handle) = &window_handle
            {
                window_handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }

            if let Some(window_handle) = &window_handle {
                let show_cursor = self
                    .world
                    .res::<crate::platform::window::Window>()
                    .show_cursor;
                window_handle.set_cursor_visible(show_cursor);
                if !self
                    .world
                    .res::<crate::platform::window::Window>()
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
                if let Err(error) =
                    crate::render_driver::configure_with_state(&mut renderer, self.state.as_mut())
                {
                    tracing::error!("Failed to configure renderer with state: {error}");
                }
                self.renderer = Some(renderer);
                self.state.initialize(&mut self.world);
                sync_min_window_size(&mut self.world);
                #[cfg(target_arch = "wasm32")]
                crate::ecs::camera::systems::sync_wasm_canvas_size(&mut self.world);

                let show_cursor = self
                    .world
                    .res::<crate::platform::window::Window>()
                    .show_cursor;
                if let Some(window_handle) =
                    &self.world.res::<crate::platform::window::Window>().handle
                {
                    window_handle.set_cursor_visible(show_cursor);
                }

                renderer_received = true;
            }
            if renderer_received {
                self.renderer_receiver = None;
            }
        }

        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            feature = "core"
        ))]
        if let Some(host) = self.accesskit.as_mut()
            && let Some(window) = self
                .world
                .res::<crate::platform::window::Window>()
                .handle
                .as_ref()
        {
            host.process_event(window, &event);
        }

        if self
            .world
            .res::<crate::platform::window::Window>()
            .should_exit
            || matches!(event, winit::event::WindowEvent::CloseRequested)
        {
            event_loop.exit();
            return;
        }

        match event {
            winit::event::WindowEvent::Focused(focused) => {
                self.world
                    .res_mut::<crate::platform::window::Window>()
                    .is_focused = focused;
                if !focused {
                    self.world
                        .res_mut::<crate::platform::input::resources::Input>()
                        .mouse
                        .position_initialized = false;
                }
            }
            winit::event::WindowEvent::Moved(_) => {
                if let Some(handle) = self
                    .world
                    .res::<crate::platform::window::Window>()
                    .handle
                    .clone()
                {
                    refresh_macos_frame_rate_limit(&mut self.world, &handle);
                }
            }
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(handle) = self
                    .world
                    .res::<crate::platform::window::Window>()
                    .handle
                    .as_ref()
                {
                    self.world
                        .res_mut::<crate::platform::window::Window>()
                        .cached_scale_factor = handle.scale_factor() as f32;
                }
                crate::platform::window::refresh_monitors_from_event_loop(
                    self.world.res_mut::<crate::platform::window::Monitors>(),
                    event_loop,
                );
                if let Some(handle) = self
                    .world
                    .res::<crate::platform::window::Window>()
                    .handle
                    .clone()
                {
                    refresh_macos_frame_rate_limit(&mut self.world, &handle);
                }
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height })
                if width > 0 && height > 0 =>
            {
                if let Some(handle) = self
                    .world
                    .res::<crate::platform::window::Window>()
                    .handle
                    .as_ref()
                {
                    self.world
                        .res_mut::<crate::platform::window::Window>()
                        .cached_scale_factor = handle.scale_factor() as f32;
                }
                crate::platform::window::refresh_monitors_from_event_loop(
                    self.world.res_mut::<crate::platform::window::Monitors>(),
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
                    self.world
                        .res_mut::<crate::platform::window::Window>()
                        .pointer_viewport_scale = (dpr / raw_dpr) as f32;
                    let physical_width = (css_width * dpr).round() as u32;
                    let physical_height = (css_height * dpr).round() as u32;
                    use winit::platform::web::WindowExtWebSys;
                    if let Some(window) = self
                        .world
                        .res::<crate::platform::window::Window>()
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
                    &mut self.world,
                    &mut self.renderer,
                    surface_width,
                    surface_height,
                );
            }
            _ => {}
        }

        if let Some(new_state) = crate::run::step(
            &mut self.world,
            &mut self.state,
            &mut self.renderer,
            &event,
            self.caller_driven_frame,
        ) {
            self.state = new_state;
        }

        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            feature = "core"
        ))]
        if let Some(host) = self.accesskit.as_mut() {
            host.update(&mut self.world);
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
            let mouse = &mut self
                .world
                .res_mut::<crate::platform::input::resources::Input>()
                .mouse;
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
    let Some(window_handle) = world
        .res_mut::<crate::platform::window::Window>()
        .handle
        .as_mut()
    else {
        return;
    };
    let winit::dpi::PhysicalSize { width, height } = window_handle.inner_size();
    if width == 0 || height == 0 {
        return;
    }
    window_handle.request_redraw();
}
