use crate::contract::{XrEye, XrHand, XrPose, XrUiPanel};
use crate::contract::{XrFrame, XrInput, XrView};
use crate::resources::session::XrUiQuad;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_wgpu::WgpuFrameRenderer;
use nightshade_wgpu::backend::WgpuRenderer;
use nightshade_window::run::pump::{PumpShell, pump_render_frame, pump_shell_with_world};

pub fn run_openxr(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = app;
    let z_near = app
        .world
        .resources
        .get::<crate::contract::XrSettings>()
        .map(|settings| settings.z_near)
        .unwrap_or(0.05);

    let (xr_instance, system, gpu) = crate::gpu::create_xr_gpu()?;
    let mut session = crate::resources::session::XrSession::new(
        xr_instance,
        system,
        &gpu,
        z_near,
        nightshade_camera::prelude::ProjectionOverride::DEFAULT_FAR,
    )?;
    let actions =
        crate::resources::actions::XrActions::new(session.handle_instance(), session.handle())?;
    let resolution = session.resolution();

    let kept_alive = app.take_kept_alive();

    if let Some(settings) = app.world.resources.get_mut::<crate::contract::XrSettings>() {
        settings.eye_resolution = Some(resolution);
    }

    let (world, host, hooks) = nightshade_wgpu::commands::into_parts(app);
    let mut shell = pump_shell_with_world(Box::new(host), world)?;

    let device = gpu.device.clone();
    let queue = gpu.queue.clone();
    let adapter = gpu.adapter.clone();
    let instance = gpu.instance.clone();
    let bindless = gpu.bindless;
    shell.context.renderer_factory = Some(Box::new(move |window, width, height| {
        Box::pin(async move {
            let surface = instance.create_surface(window)?;
            if surface.get_capabilities(&adapter).formats.is_empty() {
                return Err::<WgpuFrameRenderer, Box<dyn std::error::Error>>(
                    "the GPU the headset is attached to cannot present to the desktop window"
                        .into(),
                );
            }
            let renderer = WgpuRenderer::new_with_device(
                &adapter,
                device.clone(),
                queue.clone(),
                surface,
                nightshade_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
                bindless,
                width,
                height,
            )
            .await?;
            Ok(WgpuFrameRenderer::new(renderer, hooks))
        })
    }));

    let mut input = XrInput::default();
    let mut previous_display_time: Option<openxr::Time> = None;
    let mut applied_render_scale = 1.0_f32;

    let result = 'session: loop {
        match session.poll_events() {
            Ok(true) => {}
            Ok(false) => break 'session Ok(()),
            Err(error) => break 'session Err(error),
        }

        if shell.context.renderer.is_none() {
            if !pump_render_frame(&mut shell) {
                break 'session Ok(());
            }
            continue;
        }

        let requested_scale = shell
            .context
            .world
            .res::<nightshade_render::data::config::RenderSettings>()
            .render_scale;
        if requested_scale != applied_render_scale {
            match session.apply_resolution_scale(requested_scale, &gpu.device) {
                Ok(true) => {
                    applied_render_scale = requested_scale;
                    publish_eye_resolution(
                        &mut shell.context.world.storage,
                        &mut shell.context.world.resources,
                        session.resolution(),
                    );
                }
                Ok(false) => applied_render_scale = requested_scale,
                Err(error) => break 'session Err(error),
            }
        }

        if !session.is_running() {
            if !pump_render_frame(&mut shell) {
                break 'session Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        let frame_state = match session.wait_frame() {
            Ok(frame_state) => frame_state,
            Err(error) => break 'session Err(error),
        };

        let located = session.locate_views(frame_state.predicted_display_time);
        let (view_flags, views) = match located {
            Ok(located) => located,
            Err(error) => break 'session Err(error),
        };
        let tracking_valid = view_flags.contains(
            openxr::ViewStateFlags::POSITION_VALID | openxr::ViewStateFlags::ORIENTATION_VALID,
        );

        let frame_delta =
            display_time_delta(previous_display_time, frame_state.predicted_display_time);
        previous_display_time = Some(frame_state.predicted_display_time);
        publish_frame(
            &mut shell.context.world.resources,
            &views,
            tracking_valid,
            frame_delta,
        );

        shell.context.world.res_mut::<Time>().host_delta_time = Some(frame_delta);

        match actions.poll(session.handle(), &input) {
            Ok(polled) => input = polled,
            Err(error) => break 'session Err(error),
        }
        for hand in XrHand::ALL {
            if let Some((translation, rotation)) =
                actions.hand_pose(hand, session.stage(), frame_state.predicted_display_time)
            {
                let rig = *shell.context.world.res::<crate::contract::XrRigSlot>();
                crate::systems::frame::apply_hand_pose(
                    &rig,
                    &mut shell.context.world.storage,
                    hand,
                    translation,
                    rotation,
                );
                match hand {
                    XrHand::Left => input.left.tracked = true,
                    XrHand::Right => input.right.tracked = true,
                }
            }
            let aim = actions
                .aim_pose(hand, session.stage(), frame_state.predicted_display_time)
                .map(|(position, orientation)| XrPose {
                    position,
                    orientation,
                });
            match hand {
                XrHand::Left => input.left.aim = aim,
                XrHand::Right => input.right.aim = aim,
            }
        }
        shell.context.world.insert_resource(input);

        let panel = shell
            .context
            .world
            .resources
            .get::<XrUiPanel>()
            .copied()
            .unwrap_or_default();
        let panel_pixels = shell
            .context
            .world
            .res::<Window>()
            .cached_viewport_size
            .filter(|_| panel.enabled)
            .map(|(width, height)| (width.max(1), height.max(1)));
        let held_pixels = session.ui_swapchain_size();
        if held_pixels.is_some() && held_pixels != panel_pixels {
            release_panel_images(&mut shell, &gpu.device, &mut session);
        }
        if let Some(pixels) = panel_pixels
            && let Err(error) = session.ensure_ui_swapchain(&gpu.device, pixels)
        {
            break 'session Err(error);
        }

        if let Err(error) = session.begin_frame() {
            break 'session Err(error);
        }

        if !frame_state.should_render || !tracking_valid {
            if let Err(error) = session.abandon_frame(&frame_state) {
                break 'session Err(error);
            }
            continue;
        }

        if let Err(error) = session.acquire() {
            break 'session Err(error);
        }

        bind_eye_targets(&mut shell, &session, resolution);
        bind_ui_target(&mut shell, &session);

        if !pump_render_frame(&mut shell) {
            let _ = session.end_frame(&frame_state, &views, None);
            break 'session Ok(());
        }

        let quad = panel_pixels.map(|(pixel_width, pixel_height)| XrUiQuad {
            anchor: panel.anchor,
            distance: panel.distance,
            size: (
                panel.width,
                panel.width * pixel_height as f32 / pixel_width.max(1) as f32,
            ),
        });
        if let Err(error) = session.end_frame(&frame_state, &views, quad) {
            break 'session Err(error);
        }
    };

    tracing::info!("draining GPU work before teardown");
    let _ = gpu.device.poll(nightshade_wgpu::wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    session.shutdown();
    drop(kept_alive);

    result
}

fn bind_eye_targets(
    shell: &mut PumpShell<WgpuFrameRenderer>,
    session: &crate::resources::session::XrSession,
    resolution: (u32, u32),
) {
    let Some(rig) = shell.context.world.res::<crate::contract::XrRigSlot>().0 else {
        return;
    };
    let Some(renderer) = shell.context.renderer.as_mut() else {
        return;
    };
    let renderer = &mut renderer.renderer;
    for eye in XrEye::ALL {
        let Some((texture, view)) = session.eye_view(eye) else {
            continue;
        };
        renderer.set_external_camera_viewport(rig.eye(eye), texture, view, resolution);
        if let Some((depth, layer)) = session.eye_depth_target(eye) {
            renderer.set_external_camera_depth(rig.eye(eye), depth, layer);
        }
    }
}

fn release_panel_images(
    shell: &mut PumpShell<WgpuFrameRenderer>,
    device: &nightshade_wgpu::wgpu::Device,
    session: &mut crate::resources::session::XrSession,
) {
    if let Some(renderer) = shell.context.renderer.as_mut() {
        renderer.renderer.clear_external_paint_target();
    }
    let _ = device.poll(nightshade_wgpu::wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    session.release_ui_swapchain();
}

fn bind_ui_target(
    shell: &mut PumpShell<WgpuFrameRenderer>,
    session: &crate::resources::session::XrSession,
) {
    let Some(renderer) = shell.context.renderer.as_mut() else {
        return;
    };
    let renderer = &mut renderer.renderer;
    match session.ui_view() {
        Some((texture, view, size)) => renderer.set_external_paint_target(texture, view, size),
        None => renderer.clear_external_paint_target(),
    }
}

fn publish_eye_resolution(
    storage: &mut nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
    resolution: (u32, u32),
) {
    if let Some(settings) = resources.get_mut::<crate::contract::XrSettings>() {
        settings.eye_resolution = Some(resolution);
    }
    let Some(rig) = resources.res::<crate::contract::XrRigSlot>().0 else {
        return;
    };
    for eye in XrEye::ALL {
        storage.set(
            rig.eye(eye),
            nightshade_camera::prelude::RenderTarget::host_owned(resolution.0, resolution.1),
        );
    }
}

fn display_time_delta(previous: Option<openxr::Time>, current: openxr::Time) -> f32 {
    const NANOS_PER_SECOND: f32 = 1.0e9;
    const MAX_STEP: f32 = 0.1;
    let Some(previous) = previous else {
        return 0.0;
    };
    let elapsed = current.as_nanos().saturating_sub(previous.as_nanos());
    (elapsed as f32 / NANOS_PER_SECOND).clamp(0.0, MAX_STEP)
}

fn publish_frame(
    resources: &mut nightshade::ecs::ResourceMap,
    views: &[openxr::View],
    tracking_valid: bool,
    delta_time: f32,
) {
    let converted: Vec<XrView> = views
        .iter()
        .map(|view| XrView {
            position: crate::data::to_vec3(view.pose.position),
            orientation: crate::data::to_quat(view.pose.orientation),
            tangents: [
                view.fov.angle_left.tan(),
                view.fov.angle_right.tan(),
                view.fov.angle_up.tan(),
                view.fov.angle_down.tan(),
            ],
        })
        .collect();

    let head_position = if converted.is_empty() {
        Vec3::zeros()
    } else {
        converted
            .iter()
            .fold(Vec3::zeros(), |sum, view| sum + view.position)
            / converted.len() as f32
    };
    let head_orientation = converted
        .first()
        .map(|view| view.orientation)
        .unwrap_or_else(Quat::identity);

    resources.insert(XrFrame {
        views: converted,
        head_position,
        head_orientation,
        tracking_valid,
        delta_time,
    });
}
