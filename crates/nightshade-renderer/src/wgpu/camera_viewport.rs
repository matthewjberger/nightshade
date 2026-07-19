//! Per-camera offscreen viewport texture management.

impl super::WgpuRenderer {
    /// Ensures an offscreen viewport texture of the given size exists for
    /// `camera_entity`, creating or resizing it when absent or mismatched.
    pub fn ensure_camera_viewport(
        &mut self,
        camera_entity: nightshade_ecs::Entity,
        width: u32,
        height: u32,
    ) {
        if let Some(viewport) = self.camera_viewports.get(&camera_entity)
            && viewport.size == (width, height)
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
            },
        );
    }
}
