use winit::application::ApplicationHandler;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run_event_loop<A>(app: A) -> Result<(), winit::error::EventLoopError>
where
    A: ApplicationHandler + 'static,
{
    #[cfg(target_os = "android")]
    let event_loop = {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let android_app = nightshade_platform::android::android_app()
            .expect("android_main must call nightshade_platform::android::set_android_app first");
        crate::platform::display_area::request_whole_display();
        EventLoop::builder().with_android_app(android_app).build()?
    };

    #[cfg(not(target_os = "android"))]
    let event_loop = EventLoop::builder().build()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop.run_app(&mut app)?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }

    Ok(())
}
