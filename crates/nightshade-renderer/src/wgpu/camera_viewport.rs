//! Per-camera offscreen viewport texture management.

impl super::WgpuRenderer {
    /// Ensures an offscreen viewport texture of the given size exists for
    /// `camera_entity`, creating or resizing it when absent or mismatched.
    ///
    /// Viewports bound through [`set_external_camera_viewport`](Self::set_external_camera_viewport)
    /// are left alone: the caller owns that texture and its size.
    pub fn ensure_camera_viewport(
        &mut self,
        camera_entity: nightshade_ecs::Entity,
        width: u32,
        height: u32,
    ) {
        if let Some(viewport) = self.camera_viewports.get(&camera_entity)
            && (viewport.external || viewport.size == (width, height))
        {
            return;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Viewport Texture (Camera {})", camera_entity.id)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.camera_viewports.insert(
            camera_entity,
            super::CameraViewport {
                texture,
                view,
                size: (width, height),
                has_rendered_at_least_once: false,
                last_active_view: None,
                last_settings_version: 0,
                last_render_frame: 0,
                last_camera_world_transform: None,
                external: false,
            },
        );
    }

    /// Binds a caller-owned texture and view as `camera_entity`'s render target,
    /// so the frame renders the camera straight into it instead of into a
    /// renderer-allocated viewport.
    ///
    /// The binding carries the change-detection state of any previous viewport
    /// for the same camera, so a host that rebinds every frame (an OpenXR
    /// swapchain hands out a different image index per frame) does not reset the
    /// camera's dirty tracking and force a redundant render.
    ///
    /// Call before the frame that should use it. The renderer holds the handles
    /// only until they are replaced or the camera goes away, and never frees the
    /// underlying resource.
    pub fn set_external_camera_viewport(
        &mut self,
        camera_entity: nightshade_ecs::Entity,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        size: (u32, u32),
    ) {
        let previous = self.camera_viewports.get(&camera_entity);
        let has_rendered_at_least_once =
            previous.is_some_and(|viewport| viewport.has_rendered_at_least_once);
        let last_active_view = previous.and_then(|viewport| viewport.last_active_view);
        let last_settings_version = previous.map_or(0, |viewport| viewport.last_settings_version);
        let last_render_frame = previous.map_or(0, |viewport| viewport.last_render_frame);
        let last_camera_world_transform =
            previous.and_then(|viewport| viewport.last_camera_world_transform);

        self.camera_viewports.insert(
            camera_entity,
            super::CameraViewport {
                texture,
                view,
                size,
                has_rendered_at_least_once,
                last_active_view,
                last_settings_version,
                last_render_frame,
                last_camera_world_transform,
                external: true,
            },
        );
    }
}
