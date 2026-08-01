use crate::pre_render::PreRenderFn;
use crate::renderer::wgpu;
use crate::renderer::wgpu::pass_build::PassBuild;

pub(crate) type RenderGraphConfigFn = Box<
    dyn FnMut(
        &mut crate::renderer::wgpu::rendergraph::RenderGraph<
            crate::renderer::wgpu::render_configs::RenderInputs,
        >,
        PassBuild,
        wgpu::TextureFormat,
    ),
>;

pub(crate) type ShaderModulesFn =
    Box<dyn FnMut(&crate::renderer::wgpu::shader_compose::ShaderComposer)>;

pub(crate) type UpdateRenderGraphFn = Box<
    dyn FnMut(
        &mut crate::renderer::wgpu::rendergraph::RenderGraph<
            crate::renderer::wgpu::render_configs::RenderInputs,
        >,
        &nightshade::ecs::Storage,
        &nightshade::ecs::ResourceMap,
    ),
>;

pub(crate) type WindowEventFn = Box<
    dyn FnMut(
        &mut nightshade::ecs::Storage,
        &mut nightshade::ecs::ResourceMap,
        &nightshade_platform::winit::event::WindowEvent,
    ),
>;

#[derive(Default)]
pub struct RenderHooks {
    pub(crate) shader_modules: Vec<ShaderModulesFn>,
    pub(crate) render_graph_config: Vec<RenderGraphConfigFn>,
    pub(crate) render_graph_sub_pass: Vec<RenderGraphConfigFn>,
    pub(crate) pre_render: Vec<PreRenderFn>,
    pub(crate) window_event: Vec<WindowEventFn>,
    pub(crate) update_render_graph: Vec<UpdateRenderGraphFn>,
    pub(crate) frame_inputs: Vec<crate::renderer::wgpu::frame::FrameInputsFn>,
}

impl RenderHooks {
    pub fn publish_shader_modules(
        &mut self,
        composer: &crate::renderer::wgpu::shader_compose::ShaderComposer,
    ) {
        for hook in &mut self.shader_modules {
            hook(composer);
        }
    }

    pub fn configure_render_graph(
        &mut self,
        graph: &mut crate::renderer::wgpu::rendergraph::RenderGraph<
            crate::renderer::wgpu::render_configs::RenderInputs,
        >,
        build: PassBuild,
        format: wgpu::TextureFormat,
    ) {
        for hook in &mut self.render_graph_config {
            hook(graph, build, format);
        }
    }

    pub fn configure_render_graph_sub_passes(
        &mut self,
        graph: &mut crate::renderer::wgpu::rendergraph::RenderGraph<
            crate::renderer::wgpu::render_configs::RenderInputs,
        >,
        build: PassBuild,
        format: wgpu::TextureFormat,
    ) {
        for hook in &mut self.render_graph_sub_pass {
            hook(graph, build, format);
        }
    }

    pub fn update_render_graph(
        &mut self,
        graph: &mut crate::renderer::wgpu::rendergraph::RenderGraph<
            crate::renderer::wgpu::render_configs::RenderInputs,
        >,
        storage: &nightshade::ecs::Storage,
        resources: &nightshade::ecs::ResourceMap,
    ) {
        for hook in &mut self.update_render_graph {
            hook(graph, storage, resources);
        }
    }

    pub fn pre_render(
        &mut self,
        renderer: &mut crate::renderer::wgpu::WgpuRenderer,
        storage: &mut nightshade::ecs::Storage,
        resources: &mut nightshade::ecs::ResourceMap,
    ) {
        for hook in &mut self.pre_render {
            hook(renderer, storage, resources);
        }
    }
}
