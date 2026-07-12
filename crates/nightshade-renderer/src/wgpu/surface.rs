use super::WgpuRenderer;
use super::passes;
use super::rendergraph::render_graph_get_pass_mut;

impl WgpuRenderer {
    pub fn sync_present_mode(&mut self, vsync_enabled: bool) {
        let desired =
            super::initialization::pick_present_mode(&self.supported_present_modes, vsync_enabled);
        if desired == self.surface_config.present_mode {
            return;
        }
        self.surface_config.present_mode = desired;
        self.surface.configure(&self.device, &self.surface_config);
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
        self.surface.configure(&self.device, &self.surface_config);
        self.resize_render_buffers(width, height);

        Ok(())
    }

    pub fn register_render_texture(&mut self, name: &str, view: wgpu::TextureView) {
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{}_sampler", name)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        if let Some(mesh_pass) = render_graph_get_pass_mut(&mut self.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            mesh_pass.register_texture(name.to_string(), view, sampler);
        }
    }
}
