use crate::renderer::wgpu::RenderTargets;
use crate::renderer::wgpu::shader_compose::ShaderComposer;

#[derive(Clone, Copy)]
pub struct PassBuild<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub composer: &'a ShaderComposer,
    pub targets: &'a RenderTargets,
    pub surface_format: wgpu::TextureFormat,
    pub hdr_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
    pub point_shadow_format: wgpu::TextureFormat,
    pub size: (u32, u32),
    pub bindless: crate::renderer::wgpu::caps::BindlessConfig,
    pub low_power: bool,
}

impl crate::renderer::wgpu::WgpuRenderer {
    pub fn pass_build(&self) -> PassBuild<'_> {
        PassBuild {
            device: &self.device,
            queue: &self.queue,
            composer: &self.shader_composer,
            targets: &self.targets,
            surface_format: self.surface_format,
            hdr_format: crate::renderer::wgpu::HDR_FORMAT,
            depth_format: crate::renderer::wgpu::DEPTH_FORMAT,
            point_shadow_format: self.point_shadow_format,
            size: self.render_buffer_size,
            bindless: self.bindless,
            low_power: self.gpu_profile.is_low_power(),
        }
    }

    pub fn graph_and_pass_build(
        &mut self,
    ) -> (
        &mut crate::renderer::wgpu::rendergraph::RenderGraph<
            crate::renderer::wgpu::render_configs::RenderInputs,
        >,
        PassBuild<'_>,
    ) {
        let Self {
            graph,
            device,
            queue,
            shader_composer,
            targets,
            surface_format,
            point_shadow_format,
            render_buffer_size,
            bindless,
            gpu_profile,
            ..
        } = self;
        let surface_format = *surface_format;
        let point_shadow_format = *point_shadow_format;
        let size = *render_buffer_size;
        let bindless = *bindless;
        let low_power = gpu_profile.is_low_power();
        (
            graph,
            PassBuild {
                device,
                queue,
                composer: shader_composer,
                targets,
                surface_format,
                hdr_format: crate::renderer::wgpu::HDR_FORMAT,
                depth_format: crate::renderer::wgpu::DEPTH_FORMAT,
                point_shadow_format,
                size,
                bindless,
                low_power,
            },
        )
    }
}
