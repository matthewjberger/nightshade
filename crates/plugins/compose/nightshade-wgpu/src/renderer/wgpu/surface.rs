use super::WgpuRenderer;

impl WgpuRenderer {
    pub fn sync_present_mode(&mut self, vsync_enabled: bool) {
        let desired =
            super::initialization::pick_present_mode(&self.supported_present_modes, vsync_enabled);
        if desired == self.surface_config.present_mode {
            return;
        }
        self.surface_config.present_mode = desired;
        if let Some(surface) = self.surface.as_ref() {
            surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn resize_surface(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.surface_config.width = width;
        self.surface_config.height = height;
        if let Some(surface) = self.surface.as_ref() {
            surface.configure(&self.device, &self.surface_config);
        }
        self.resize_render_buffers(width, height);

        Ok(())
    }

    pub fn release_surface(&mut self) {
        self.surface = None;
    }

    pub fn recreate_surface(
        &mut self,
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(instance) = self.instance.as_ref() else {
            return Err("this renderer has no instance to build a surface from".into());
        };

        let surface = instance.create_surface(window)?;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        surface.configure(&self.device, &self.surface_config);
        self.surface = Some(surface);
        self.resize_render_buffers(width.max(1), height.max(1));

        Ok(())
    }
}
