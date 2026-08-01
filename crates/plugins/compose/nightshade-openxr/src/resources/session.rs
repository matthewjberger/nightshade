use ash::vk;
use openxr as xr;

const SWAPCHAIN_FORMAT: nightshade_wgpu::wgpu::TextureFormat =
    nightshade_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb;

const DEPTH_FORMAT: nightshade_wgpu::wgpu::TextureFormat =
    nightshade_wgpu::wgpu::TextureFormat::Depth32Float;

const EYE_LAYERS: u32 = 2;

pub struct XrUiQuad {
    pub anchor: crate::contract::XrUiAnchor,
    pub distance: f32,
    pub size: (f32, f32),
}

pub struct XrSession {
    instance: xr::Instance,
    session: xr::Session<xr::Vulkan>,
    frame_wait: xr::FrameWaiter,
    frame_stream: xr::FrameStream<xr::Vulkan>,
    stage: xr::Space,
    view_space: xr::Space,
    swapchain: xr::Swapchain<xr::Vulkan>,
    images: Vec<nightshade_wgpu::wgpu::Texture>,
    depth: Option<DepthSwapchain>,
    ui: Option<UiSwapchain>,
    recommended: (u32, u32),
    resolution: (u32, u32),
    running: bool,
    frame_open: bool,
    acquired: Option<u32>,
    z_near: f32,
    z_far: f32,
}

struct DepthSwapchain {
    swapchain: xr::Swapchain<xr::Vulkan>,
    images: Vec<nightshade_wgpu::wgpu::Texture>,
    acquired: Option<u32>,
}

struct UiSwapchain {
    swapchain: xr::Swapchain<xr::Vulkan>,
    images: Vec<nightshade_wgpu::wgpu::Texture>,
    size: (u32, u32),
    acquired: Option<u32>,
}

impl XrSession {
    pub fn new(
        instance: xr::Instance,
        system: xr::SystemId,
        gpu: &crate::gpu::XrGpu,
        z_near: f32,
        z_far: f32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let views = instance.enumerate_view_configuration_views(
            system,
            xr::ViewConfigurationType::PRIMARY_STEREO,
        )?;
        let resolution = (
            views[0].recommended_image_rect_width,
            views[0].recommended_image_rect_height,
        );

        let (session, frame_wait, frame_stream) =
            unsafe { instance.create_session::<xr::Vulkan>(system, &gpu.session_create_info) }?;

        let stage = if gpu.local_floor_supported {
            match session
                .create_reference_space(xr::ReferenceSpaceType::LOCAL_FLOOR, xr::Posef::IDENTITY)
            {
                Ok(space) => {
                    tracing::info!("using the runtime's derived floor for the play space");
                    space
                }
                Err(error) => {
                    tracing::warn!("no derived floor, falling back to the stage: {error}");
                    session.create_reference_space(
                        xr::ReferenceSpaceType::STAGE,
                        xr::Posef::IDENTITY,
                    )?
                }
            }
        } else {
            session.create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)?
        };

        let view_space =
            session.create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)?;

        let (swapchain, images) =
            create_color_swapchain(&session, &gpu.device, resolution, EYE_LAYERS)?;

        let depth = if gpu.depth_layer_supported {
            match create_depth_swapchain(&session, &gpu.device, resolution) {
                Ok(depth) => Some(depth),
                Err(error) => {
                    tracing::warn!("no depth swapchain, submitting color only: {error}");
                    None
                }
            }
        } else {
            None
        };

        tracing::info!(
            "OpenXR session created, {}x{} per eye, {} swapchain images, depth layer {}",
            resolution.0,
            resolution.1,
            images.len(),
            if depth.is_some() { "on" } else { "off" }
        );

        Ok(Self {
            instance,
            session,
            frame_wait,
            frame_stream,
            stage,
            view_space,
            swapchain,
            images,
            depth,
            ui: None,
            recommended: resolution,
            resolution,
            running: false,
            frame_open: false,
            acquired: None,
            z_near,
            z_far,
        })
    }

    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    pub fn apply_resolution_scale(
        &mut self,
        scale: f32,
        device: &nightshade_wgpu::wgpu::Device,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let scale = scale.clamp(0.25, 2.0);
        let target = (
            ((self.recommended.0 as f32 * scale).round() as u32).max(1),
            ((self.recommended.1 as f32 * scale).round() as u32).max(1),
        );
        if target == self.resolution {
            return Ok(false);
        }

        let (swapchain, images) =
            create_color_swapchain(&self.session, device, target, EYE_LAYERS)?;
        self.swapchain = swapchain;
        self.images = images;
        self.acquired = None;

        if self.depth.is_some() {
            self.depth = match create_depth_swapchain(&self.session, device, target) {
                Ok(depth) => Some(depth),
                Err(error) => {
                    tracing::warn!(
                        "no depth swapchain after resize, submitting color only: {error}"
                    );
                    None
                }
            };
        }

        self.resolution = target;
        Ok(true)
    }

    pub fn ensure_ui_swapchain(
        &mut self,
        device: &nightshade_wgpu::wgpu::Device,
        size: (u32, u32),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = (size.0.max(1), size.1.max(1));
        if self.ui.as_ref().is_some_and(|ui| ui.size == size) {
            return Ok(());
        }
        let (swapchain, images) = create_color_swapchain(&self.session, device, size, 1)?;
        tracing::info!("OpenXR UI swapchain created, {}x{}", size.0, size.1);
        self.ui = Some(UiSwapchain {
            swapchain,
            images,
            size,
            acquired: None,
        });
        Ok(())
    }

    pub fn ui_swapchain_size(&self) -> Option<(u32, u32)> {
        self.ui.as_ref().map(|ui| ui.size)
    }

    pub fn release_ui_swapchain(&mut self) {
        self.ui = None;
    }

    pub fn handle(&self) -> &xr::Session<xr::Vulkan> {
        &self.session
    }

    pub fn handle_instance(&self) -> &xr::Instance {
        &self.instance
    }

    pub fn stage(&self) -> &xr::Space {
        &self.stage
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn poll_events(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut buffer = xr::EventDataBuffer::new();
        while let Some(event) = self.instance.poll_event(&mut buffer)? {
            match event {
                xr::Event::SessionStateChanged(changed) => {
                    tracing::info!("OpenXR session state: {:?}", changed.state());
                    match changed.state() {
                        xr::SessionState::READY => {
                            self.session
                                .begin(xr::ViewConfigurationType::PRIMARY_STEREO)?;
                            self.running = true;
                        }
                        xr::SessionState::STOPPING => {
                            self.running = false;
                            self.session.end()?;
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            self.running = false;
                            return Ok(false);
                        }
                        _ => {}
                    }
                }
                xr::Event::InstanceLossPending(_) => return Ok(false),
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn wait_frame(&mut self) -> Result<xr::FrameState, Box<dyn std::error::Error>> {
        Ok(self.frame_wait.wait()?)
    }

    pub fn locate_views(
        &self,
        display_time: xr::Time,
    ) -> Result<(xr::ViewStateFlags, Vec<xr::View>), Box<dyn std::error::Error>> {
        Ok(self.session.locate_views(
            xr::ViewConfigurationType::PRIMARY_STEREO,
            display_time,
            &self.stage,
        )?)
    }

    pub fn begin_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.frame_stream.begin()?;
        self.frame_open = true;
        Ok(())
    }

    pub fn abandon_frame(
        &mut self,
        frame_state: &xr::FrameState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.frame_open {
            return Ok(());
        }
        self.frame_stream.end(
            frame_state.predicted_display_time,
            xr::EnvironmentBlendMode::OPAQUE,
            &[],
        )?;
        self.frame_open = false;
        Ok(())
    }

    pub fn acquire(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let index = self.swapchain.acquire_image()?;
        self.swapchain.wait_image(xr::Duration::INFINITE)?;
        self.acquired = Some(index);

        if let Some(depth) = self.depth.as_mut() {
            let index = depth.swapchain.acquire_image()?;
            depth.swapchain.wait_image(xr::Duration::INFINITE)?;
            depth.acquired = Some(index);
        }
        if let Some(ui) = self.ui.as_mut() {
            let index = ui.swapchain.acquire_image()?;
            ui.swapchain.wait_image(xr::Duration::INFINITE)?;
            ui.acquired = Some(index);
        }
        Ok(())
    }

    pub fn ui_view(
        &self,
    ) -> Option<(
        nightshade_wgpu::wgpu::Texture,
        nightshade_wgpu::wgpu::TextureView,
        (u32, u32),
    )> {
        let ui = self.ui.as_ref()?;
        let texture = ui.images.get(ui.acquired? as usize)?;
        let view = texture.create_view(&nightshade_wgpu::wgpu::TextureViewDescriptor {
            label: Some("OpenXR UI"),
            format: Some(SWAPCHAIN_FORMAT),
            dimension: Some(nightshade_wgpu::wgpu::TextureViewDimension::D2),
            aspect: nightshade_wgpu::wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(1),
            usage: None,
        });
        Some((texture.clone(), view, ui.size))
    }

    pub fn eye_depth_target(
        &self,
        eye: crate::contract::XrEye,
    ) -> Option<(nightshade_wgpu::wgpu::Texture, u32)> {
        let depth = self.depth.as_ref()?;
        let texture = depth.images.get(depth.acquired? as usize)?;
        Some((texture.clone(), eye.layer()))
    }

    pub fn eye_view(
        &self,
        eye: crate::contract::XrEye,
    ) -> Option<(
        nightshade_wgpu::wgpu::Texture,
        nightshade_wgpu::wgpu::TextureView,
    )> {
        let texture = self.images.get(self.acquired? as usize)?;
        let view = texture.create_view(&nightshade_wgpu::wgpu::TextureViewDescriptor {
            label: Some("OpenXR eye"),
            format: Some(SWAPCHAIN_FORMAT),
            dimension: Some(nightshade_wgpu::wgpu::TextureViewDimension::D2),
            aspect: nightshade_wgpu::wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: eye.layer(),
            array_layer_count: Some(1),
            usage: None,
        });
        Some((texture.clone(), view))
    }

    pub fn end_frame(
        &mut self,
        frame_state: &xr::FrameState,
        views: &[xr::View],
        quad: Option<XrUiQuad>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.acquired.take().is_some() {
            self.swapchain.release_image()?;
        }
        if let Some(depth) = self.depth.as_mut()
            && depth.acquired.take().is_some()
        {
            depth.swapchain.release_image()?;
        }
        let ui_released = match self.ui.as_mut() {
            Some(ui) => {
                let released = ui.acquired.take().is_some();
                if released {
                    ui.swapchain.release_image()?;
                }
                released
            }
            None => false,
        };

        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di {
                width: self.resolution.0 as i32,
                height: self.resolution.1 as i32,
            },
        };

        let depth_infos: Vec<openxr::sys::CompositionLayerDepthInfoKHR> = match self.depth.as_ref()
        {
            Some(depth) => views
                .iter()
                .enumerate()
                .map(|(index, _)| openxr::sys::CompositionLayerDepthInfoKHR {
                    ty: openxr::sys::CompositionLayerDepthInfoKHR::TYPE,
                    next: std::ptr::null(),
                    sub_image: openxr::sys::SwapchainSubImage {
                        swapchain: depth.swapchain.as_raw(),
                        image_rect: rect,
                        image_array_index: index as u32,
                    },
                    min_depth: 0.0,
                    max_depth: 1.0,
                    near_z: self.z_far,
                    far_z: self.z_near,
                })
                .collect(),
            None => Vec::new(),
        };

        let projection_views: Vec<_> = views
            .iter()
            .enumerate()
            .map(|(index, view)| {
                let projection_view = xr::CompositionLayerProjectionView::new()
                    .pose(view.pose)
                    .fov(view.fov)
                    .sub_image(
                        xr::SwapchainSubImage::new()
                            .swapchain(&self.swapchain)
                            .image_array_index(index as u32)
                            .image_rect(rect),
                    );
                let Some(depth_info) = depth_infos.get(index) else {
                    return projection_view;
                };
                let mut raw = projection_view.into_raw();
                raw.next = depth_info as *const _ as *const std::ffi::c_void;
                unsafe { xr::CompositionLayerProjectionView::from_raw(raw) }
            })
            .collect();

        let layer = xr::CompositionLayerProjection::new()
            .space(&self.stage)
            .views(&projection_views);

        let quad_layer = match (quad, self.ui.as_ref()) {
            (Some(quad), Some(ui)) if ui_released => {
                let (space, pose) = match quad.anchor {
                    crate::contract::XrUiAnchor::Head => (
                        &self.view_space,
                        xr::Posef {
                            orientation: xr::Quaternionf::IDENTITY,
                            position: xr::Vector3f {
                                x: 0.0,
                                y: 0.0,
                                z: -quad.distance,
                            },
                        },
                    ),
                    crate::contract::XrUiAnchor::Stage(pose) => (&self.stage, to_posef(pose)),
                };
                Some(
                    xr::CompositionLayerQuad::new()
                        .layer_flags(xr::CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA)
                        .space(space)
                        .eye_visibility(xr::EyeVisibility::BOTH)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&ui.swapchain)
                                .image_array_index(0)
                                .image_rect(xr::Rect2Di {
                                    offset: xr::Offset2Di { x: 0, y: 0 },
                                    extent: xr::Extent2Di {
                                        width: ui.size.0 as i32,
                                        height: ui.size.1 as i32,
                                    },
                                }),
                        )
                        .pose(pose)
                        .size(xr::Extent2Df {
                            width: quad.size.0,
                            height: quad.size.1,
                        }),
                )
            }
            _ => None,
        };

        let mut layers: Vec<&xr::CompositionLayerBase<'_, xr::Vulkan>> = vec![&layer];
        if let Some(quad_layer) = quad_layer.as_ref() {
            layers.push(quad_layer);
        }

        self.frame_stream.end(
            frame_state.predicted_display_time,
            xr::EnvironmentBlendMode::OPAQUE,
            &layers,
        )?;
        self.frame_open = false;
        Ok(())
    }

    pub fn shutdown(&mut self) {
        if !self.running {
            return;
        }
        if let Err(error) = self.session.request_exit() {
            tracing::warn!("xrRequestExitSession failed during shutdown: {error}");
            self.running = false;
            return;
        }

        let mut buffer = xr::EventDataBuffer::new();
        for _ in 0..200 {
            match self.instance.poll_event(&mut buffer) {
                Ok(Some(xr::Event::SessionStateChanged(changed))) => match changed.state() {
                    xr::SessionState::STOPPING => {
                        if let Err(error) = self.session.end() {
                            tracing::warn!("xrEndSession failed during shutdown: {error}");
                            break;
                        }
                    }
                    xr::SessionState::EXITING => break,
                    _ => {}
                },
                Ok(Some(_)) => {}
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
                Err(error) => {
                    tracing::warn!("event polling failed during shutdown: {error}");
                    break;
                }
            }
        }
        self.running = false;
    }
}

fn to_posef(pose: crate::contract::XrPose) -> xr::Posef {
    xr::Posef {
        orientation: xr::Quaternionf {
            x: pose.orientation.i,
            y: pose.orientation.j,
            z: pose.orientation.k,
            w: pose.orientation.w,
        },
        position: xr::Vector3f {
            x: pose.position.x,
            y: pose.position.y,
            z: pose.position.z,
        },
    }
}

fn create_color_swapchain(
    session: &xr::Session<xr::Vulkan>,
    device: &nightshade_wgpu::wgpu::Device,
    resolution: (u32, u32),
    layers: u32,
) -> Result<
    (
        xr::Swapchain<xr::Vulkan>,
        Vec<nightshade_wgpu::wgpu::Texture>,
    ),
    Box<dyn std::error::Error>,
> {
    let swapchain = session.create_swapchain(&xr::SwapchainCreateInfo {
        create_flags: xr::SwapchainCreateFlags::EMPTY,
        usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT | xr::SwapchainUsageFlags::SAMPLED,
        format: vk::Format::R8G8B8A8_SRGB.as_raw() as _,
        sample_count: 1,
        width: resolution.0,
        height: resolution.1,
        face_count: 1,
        array_size: layers,
        mip_count: 1,
    })?;

    let images = swapchain
        .enumerate_images()?
        .into_iter()
        .map(|raw_image| {
            wrap_image(
                device,
                raw_image,
                extent(resolution, layers),
                SWAPCHAIN_FORMAT,
                nightshade_wgpu::wgpu::TextureUses::COLOR_TARGET
                    | nightshade_wgpu::wgpu::TextureUses::COPY_DST,
                nightshade_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT
                    | nightshade_wgpu::wgpu::TextureUsages::COPY_DST,
                "OpenXR swapchain",
            )
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    Ok((swapchain, images))
}

fn create_depth_swapchain(
    session: &xr::Session<xr::Vulkan>,
    device: &nightshade_wgpu::wgpu::Device,
    resolution: (u32, u32),
) -> Result<DepthSwapchain, Box<dyn std::error::Error>> {
    let swapchain = session.create_swapchain(&xr::SwapchainCreateInfo {
        create_flags: xr::SwapchainCreateFlags::EMPTY,
        usage_flags: xr::SwapchainUsageFlags::DEPTH_STENCIL_ATTACHMENT
            | xr::SwapchainUsageFlags::TRANSFER_DST,
        format: vk::Format::D32_SFLOAT.as_raw() as _,
        sample_count: 1,
        width: resolution.0,
        height: resolution.1,
        face_count: 1,
        array_size: 2,
        mip_count: 1,
    })?;

    let images = swapchain
        .enumerate_images()?
        .into_iter()
        .map(|raw_image| {
            wrap_image(
                device,
                raw_image,
                extent(resolution, EYE_LAYERS),
                DEPTH_FORMAT,
                nightshade_wgpu::wgpu::TextureUses::DEPTH_STENCIL_WRITE
                    | nightshade_wgpu::wgpu::TextureUses::COPY_DST,
                nightshade_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT
                    | nightshade_wgpu::wgpu::TextureUsages::COPY_DST,
                "OpenXR depth swapchain",
            )
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    Ok(DepthSwapchain {
        swapchain,
        images,
        acquired: None,
    })
}

fn extent(resolution: (u32, u32), layers: u32) -> nightshade_wgpu::wgpu::Extent3d {
    nightshade_wgpu::wgpu::Extent3d {
        width: resolution.0,
        height: resolution.1,
        depth_or_array_layers: layers,
    }
}

fn wrap_image(
    device: &nightshade_wgpu::wgpu::Device,
    raw_image: u64,
    size: nightshade_wgpu::wgpu::Extent3d,
    format: nightshade_wgpu::wgpu::TextureFormat,
    hal_usage: nightshade_wgpu::wgpu::TextureUses,
    usage: nightshade_wgpu::wgpu::TextureUsages,
    label: &str,
) -> Result<nightshade_wgpu::wgpu::Texture, Box<dyn std::error::Error>> {
    use ash::vk::Handle;

    let hal_texture = unsafe {
        let hal_device = device
            .as_hal::<wgpu_hal::vulkan::Api>()
            .ok_or("the OpenXR device is not a Vulkan device")?;
        hal_device.texture_from_raw(
            vk::Image::from_raw(raw_image),
            &wgpu_hal::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: nightshade_wgpu::wgpu::TextureDimension::D2,
                format,
                usage: hal_usage,
                memory_flags: wgpu_hal::MemoryFlags::empty(),
                view_formats: vec![],
            },
            None,
            wgpu_hal::vulkan::TextureMemory::External,
        )
    };

    Ok(unsafe {
        device.create_texture_from_hal::<wgpu_hal::vulkan::Api>(
            hal_texture,
            &nightshade_wgpu::wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: nightshade_wgpu::wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            },
        )
    })
}
