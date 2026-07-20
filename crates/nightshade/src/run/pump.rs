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
