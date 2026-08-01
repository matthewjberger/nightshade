use crate::pre_render::IntoPreRenderSystem;
use crate::renderer::wgpu;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::resources::{RenderHooks, WindowEventFn};
use nightshade::World;
use nightshade::app::App;

pub trait RenderAppExt {
    fn add_render_graph_config(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::rendergraph::RenderGraph<
                crate::renderer::wgpu::render_configs::RenderInputs,
            >,
            PassBuild,
            wgpu::TextureFormat,
        ) + 'static,
    ) -> &mut Self;

    fn add_render_graph_sub_pass(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::rendergraph::RenderGraph<
                crate::renderer::wgpu::render_configs::RenderInputs,
            >,
            PassBuild,
            wgpu::TextureFormat,
        ) + 'static,
    ) -> &mut Self;

    fn on_window_event(
        &mut self,
        hook: impl FnMut(
            &mut nightshade::ecs::Storage,
            &mut nightshade::ecs::ResourceMap,
            &nightshade_platform::winit::event::WindowEvent,
        ) + 'static,
    ) -> &mut Self;

    fn on_pre_render<Marker>(
        &mut self,
        system: impl IntoPreRenderSystem<wgpu::WgpuRenderer, Marker>,
    ) -> &mut Self;

    fn add_shader_modules(
        &mut self,
        hook: impl FnMut(&crate::renderer::wgpu::shader_compose::ShaderComposer) + 'static,
    ) -> &mut Self;

    fn on_frame_inputs(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::WgpuRenderer,
            &mut crate::renderer::wgpu::render_configs::RenderInputs,
        ) + 'static,
    ) -> &mut Self;

    fn on_update_render_graph(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::rendergraph::RenderGraph<
                crate::renderer::wgpu::render_configs::RenderInputs,
            >,
            &nightshade::ecs::Storage,
            &nightshade::ecs::ResourceMap,
        ) + 'static,
    ) -> &mut Self;
}

impl RenderAppExt for App {
    fn add_render_graph_config(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::rendergraph::RenderGraph<
                crate::renderer::wgpu::render_configs::RenderInputs,
            >,
            PassBuild,
            wgpu::TextureFormat,
        ) + 'static,
    ) -> &mut Self {
        hooks_mut(self).render_graph_config.push(Box::new(hook));
        self
    }

    fn add_render_graph_sub_pass(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::rendergraph::RenderGraph<
                crate::renderer::wgpu::render_configs::RenderInputs,
            >,
            PassBuild,
            wgpu::TextureFormat,
        ) + 'static,
    ) -> &mut Self {
        hooks_mut(self).render_graph_sub_pass.push(Box::new(hook));
        self
    }

    fn on_window_event(
        &mut self,
        hook: impl FnMut(
            &mut nightshade::ecs::Storage,
            &mut nightshade::ecs::ResourceMap,
            &nightshade_platform::winit::event::WindowEvent,
        ) + 'static,
    ) -> &mut Self {
        hooks_mut(self).window_event.push(Box::new(hook));
        self
    }

    fn on_pre_render<Marker>(
        &mut self,
        system: impl IntoPreRenderSystem<wgpu::WgpuRenderer, Marker>,
    ) -> &mut Self {
        hooks_mut(self)
            .pre_render
            .push(system.into_pre_render_runner());
        self
    }

    fn add_shader_modules(
        &mut self,
        hook: impl FnMut(&crate::renderer::wgpu::shader_compose::ShaderComposer) + 'static,
    ) -> &mut Self {
        hooks_mut(self).shader_modules.push(Box::new(hook));
        self
    }

    fn on_frame_inputs(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::WgpuRenderer,
            &mut crate::renderer::wgpu::render_configs::RenderInputs,
        ) + 'static,
    ) -> &mut Self {
        hooks_mut(self).frame_inputs.push(Box::new(hook));
        self
    }

    fn on_update_render_graph(
        &mut self,
        hook: impl FnMut(
            &mut crate::renderer::wgpu::rendergraph::RenderGraph<
                crate::renderer::wgpu::render_configs::RenderInputs,
            >,
            &nightshade::ecs::Storage,
            &nightshade::ecs::ResourceMap,
        ) + 'static,
    ) -> &mut Self {
        hooks_mut(self).update_render_graph.push(Box::new(hook));
        self
    }
}

fn hooks_mut(app: &mut App) -> &mut RenderHooks {
    let slot = app.plugin_part();
    if slot.is_none() {
        *slot = Some(Box::new(RenderHooks::default()));
    }
    slot.as_mut()
        .and_then(|hooks| hooks.downcast_mut::<RenderHooks>())
        .expect("render hooks slot holds RenderHooks")
}

pub fn into_parts(app: App) -> (World, StageState, RenderHooks) {
    let mut app = app;
    let hooks = app
        .plugin_part()
        .take()
        .and_then(|hooks| hooks.downcast::<RenderHooks>().ok())
        .map(|hooks| *hooks)
        .unwrap_or_default();
    let mut hooks = hooks;
    let window_event = std::mem::take(&mut hooks.window_event);
    let (world, startup, stages) = app.into_schedules();
    (
        world,
        StageState {
            startup,
            stages,
            window_event,
        },
        hooks,
    )
}

pub struct StageState {
    startup: nightshade::ecs::Schedule<World>,
    stages: nightshade::ecs::Stages<World>,
    window_event: Vec<WindowEventFn>,
}

impl nightshade_window::Host for StageState {
    fn initialize(&mut self, world: &mut World) {
        self.startup.run(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        self.stages.run(world);
    }

    fn window_event(
        &mut self,
        storage: &mut nightshade::ecs::Storage,
        resources: &mut nightshade::ecs::ResourceMap,
        event: &nightshade_platform::winit::event::WindowEvent,
    ) {
        for hook in &mut self.window_event {
            hook(storage, resources, event);
        }
    }
}
