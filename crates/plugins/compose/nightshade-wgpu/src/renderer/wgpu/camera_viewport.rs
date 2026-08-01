use crate::renderer::wgpu::build;
impl super::WgpuRenderer {
    pub fn ensure_camera_viewport(
        &mut self,
        camera_entity: nightshade::ecs::Entity,
        width: u32,
        height: u32,
    ) {
        let (width, height) = self.clamp_render_extent(width, height);
        if let Some(viewport) = self.camera_viewports.get(&camera_entity)
            && viewport.size == (width, height)
        {
            return;
        }

        let texture = build::surface(
            &self.device,
            &format!("Viewport Texture (Camera {})", camera_entity.id),
            self.surface_format,
            [width, height, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

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
                depth_target: None,
                last_view_projection: None,
            },
        );
    }

    pub fn set_external_camera_viewport(
        &mut self,
        camera_entity: nightshade::ecs::Entity,
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
        let depth_target = previous.and_then(|viewport| viewport.depth_target.clone());
        let last_view_projection = previous.and_then(|viewport| viewport.last_view_projection);

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
                depth_target,
                last_view_projection,
            },
        );
    }

    pub fn set_external_camera_depth(
        &mut self,
        camera_entity: nightshade::ecs::Entity,
        texture: wgpu::Texture,
        array_layer: u32,
    ) {
        if let Some(viewport) = self.camera_viewports.get_mut(&camera_entity) {
            viewport.depth_target = Some((texture, array_layer));
        }
    }
}
