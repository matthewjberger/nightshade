use super::WindowContext;
use crate::host::Host;
use crate::renderer::Renderer;
use nightshade::World;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};

pub struct PumpShell<R: Renderer> {
    event_loop: EventLoop<()>,
    pub context: WindowContext<R>,
}

pub fn pump_shell_new<R: Renderer>(
    host: Box<dyn Host>,
) -> Result<PumpShell<R>, Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().try_init().ok();

    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let context = WindowContext {
        host,
        world: World::tracking(),
        renderer: None,
        initialized: false,
        renderer_factory: None,
        caller_driven_frame: false,
    };

    Ok(PumpShell {
        event_loop,
        context,
    })
}

pub fn pump_shell_with_world<R: Renderer>(
    host: Box<dyn Host>,
    world: World,
) -> Result<PumpShell<R>, Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let context = WindowContext {
        host,
        world,
        renderer: None,
        initialized: false,
        renderer_factory: None,
        caller_driven_frame: false,
    };

    Ok(PumpShell {
        event_loop,
        context,
    })
}

pub fn pump_shell_ready<R: Renderer>(shell: &PumpShell<R>) -> bool {
    shell.context.initialized && shell.context.renderer.is_some()
}

pub fn pump_frame<R: Renderer + 'static>(shell: &mut PumpShell<R>) -> bool {
    shell.context.caller_driven_frame = false;

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
            .res::<nightshade_platform::contract::window::Window>()
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

pub fn pump_render_frame<R: Renderer + 'static>(shell: &mut PumpShell<R>) -> bool {
    shell.context.caller_driven_frame = true;

    let status = shell
        .event_loop
        .pump_app_events(Some(std::time::Duration::ZERO), &mut shell.context);
    if matches!(status, PumpStatus::Exit(_)) {
        return false;
    }

    if shell.context.renderer.is_none() {
        return true;
    }

    crate::run::advance_timing(&mut shell.context.world.resources);

    let Some(renderer) = shell.context.renderer.as_mut() else {
        return true;
    };

    if let Some(next_host) = crate::run::run_frame_body(
        &mut shell.context.world,
        shell.context.host.as_mut(),
        renderer,
    ) {
        shell.context.host = next_host;
    }

    true
}
