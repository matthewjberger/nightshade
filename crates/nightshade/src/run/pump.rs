//! Caller-owned main loop.
//!
//! [`launch`](crate::run::launch) owns the winit event loop and drives your
//! [`State`] through callbacks. This module inverts that: you own the loop and
//! pump the engine one frame at a time. Window creation, renderer setup,
//! suspend and resume, frame pacing, and exit handling all run through the
//! same [`WindowContext`] handler that `launch` uses, so there is one driver,
//! not two.
//!
//! ```ignore
//! let mut shell = pump_shell_new(Box::new(MyState))?;
//! while pump_frame(&mut shell) {
//!     let world = &mut shell.context.world;
//! }
//! ```
//!
//! Native only. The browser owns the loop on wasm, so use [`launch`] there.
//! Logging is console only here. File logging stays a [`launch`] feature.

use super::WindowContext;
use crate::ecs::world::World;
use crate::state::State;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};

/// The event loop and engine driver for a caller-owned main loop.
///
/// `context` is the same handler `launch` drives. Its `world` field is public,
/// so callers read and mutate engine state directly between pumps.
pub struct PumpShell {
    event_loop: EventLoop<()>,
    pub context: WindowContext,
}

/// Creates the event loop and an uninitialized [`WindowContext`] around `state`.
///
/// The window and renderer do not exist yet. Pump with [`pump_frame`] until
/// [`pump_shell_ready`] returns true. The first pump delivers the resume event,
/// which creates the window, builds the renderer, and calls
/// `state.initialize`.
pub fn pump_shell_new(state: Box<dyn State>) -> Result<PumpShell, Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().try_init().ok();

    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let context = WindowContext {
        state,
        world: World::default(),
        renderer: None,
        #[cfg(all(not(target_os = "android"), feature = "core"))]
        accesskit: None,
        initialized: false,
        renderer_factory: None,
        caller_driven_frame: false,
    };

    Ok(PumpShell {
        event_loop,
        context,
    })
}

/// Creates the event loop and an uninitialized [`WindowContext`] around `state`
/// and a world the caller already built.
///
/// [`pump_shell_new`] starts from `World::default()`, which discards the member
/// worlds, resources, and settings a plugin-composed
/// [`App`](crate::app::App) registered. A runner installed with
/// [`App::set_runner`](crate::app::App::set_runner) splits the app with
/// `into_parts` and brings both halves here, the same way
/// [`launch_with_world`](crate::run::launch_with_world) does for the
/// engine-owned loop.
///
/// Unlike [`pump_shell_new`] this installs no tracing subscriber, because a
/// composed app's `LogPlugin` already owns that.
pub fn pump_shell_with_world(
    state: Box<dyn State>,
    world: World,
) -> Result<PumpShell, Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let context = WindowContext {
        state,
        world,
        renderer: None,
        #[cfg(all(not(target_os = "android"), feature = "core"))]
        accesskit: None,
        initialized: false,
        renderer_factory: None,
        caller_driven_frame: false,
    };

    Ok(PumpShell {
        event_loop,
        context,
    })
}

/// True once the window and renderer exist and `state.initialize` has run.
pub fn pump_shell_ready(shell: &PumpShell) -> bool {
    shell.context.initialized && shell.context.renderer.is_some()
}

/// Pumps pending window events, rendering at most one frame.
///
/// Returns false once the event loop has exited, from a close request or
/// `world.res::<crate::platform::window::Window>().should_exit`. Sleeps briefly when nothing can
/// render (no window or renderer yet, or a zero-sized window) so a caller
/// loop never spins hot.
pub fn pump_frame(shell: &mut PumpShell) -> bool {
    let status = shell
        .event_loop
        .pump_app_events(Some(std::time::Duration::ZERO), &mut shell.context);
    if matches!(status, PumpStatus::Exit(_)) {
        return false;
    }

    let renderable = shell.context.renderer.is_some()
        && shell
            .context
            .world
            .res::<crate::platform::window::Window>()
            .handle
            .as_ref()
            .is_some_and(|handle| {
                let size = handle.inner_size();
                size.width > 0 && size.height > 0
            });
    if !renderable {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    true
}

/// Pumps pending window events and then renders one frame unconditionally,
/// rather than waiting for the window to ask for a redraw.
///
/// Returns false once the event loop has exited, matching [`pump_frame`].
/// Renders nothing and reports true when the renderer does not exist yet.
///
/// [`pump_frame`] renders only on `RedrawRequested`, which the window asks for
/// through `about_to_wait`, and that request is skipped while the window reports
/// a zero size. A minimized or hidden window therefore stops producing frames.
/// That is the right behavior when the window is the output, and the wrong one
/// when it is not: a loop presenting somewhere else, such as an OpenXR
/// swapchain, must keep rendering while the desktop window is minimized, and
/// its pacing comes from that other presentation path rather than from winit.
pub fn pump_render_frame(shell: &mut PumpShell) -> bool {
    // Claim the frame before pumping, so the redraw request the window raises
    // during this pump is not answered by running a second full frame.
    shell.context.caller_driven_frame = true;

    let status = shell
        .event_loop
        .pump_app_events(Some(std::time::Duration::ZERO), &mut shell.context);
    if matches!(status, PumpStatus::Exit(_)) {
        return false;
    }

    let Some(renderer) = shell.context.renderer.as_mut() else {
        return true;
    };

    if let Some(next_state) = crate::run::run_frame_body(
        &mut shell.context.world,
        shell.context.state.as_mut(),
        renderer,
    ) {
        shell.context.state = next_state;
    }

    true
}
