use crate::renderer::VulkanRenderer;
use crate::renderer::pass::Pass;
use crate::resources::{RenderHooks, StageState};
use nightshade::World;
use nightshade::app::App;
use nightshade::ecs::{ResourceMap, Storage};

pub trait VulkanAppExt {
    fn install_vulkan_pass(&mut self, hook: impl FnMut(&mut VulkanRenderer) + 'static)
    -> &mut Self;

    fn on_vulkan_pre_render<Marker>(
        &mut self,
        system: impl crate::pre_render::IntoPreRenderSystem<VulkanRenderer, Marker>,
    ) -> &mut Self;

    fn on_vulkan_window_event(
        &mut self,
        hook: impl FnMut(
            &mut Storage,
            &mut ResourceMap,
            &nightshade_platform::winit::event::WindowEvent,
        ) + 'static,
    ) -> &mut Self;
}

impl VulkanAppExt for App {
    fn install_vulkan_pass(
        &mut self,
        hook: impl FnMut(&mut VulkanRenderer) + 'static,
    ) -> &mut Self {
        hooks_mut(self).install.push(Box::new(hook));
        self
    }

    fn on_vulkan_pre_render<Marker>(
        &mut self,
        system: impl crate::pre_render::IntoPreRenderSystem<VulkanRenderer, Marker>,
    ) -> &mut Self {
        hooks_mut(self)
            .pre_render
            .push(system.into_pre_render_runner());
        self
    }

    fn on_vulkan_window_event(
        &mut self,
        hook: impl FnMut(
            &mut Storage,
            &mut ResourceMap,
            &nightshade_platform::winit::event::WindowEvent,
        ) + 'static,
    ) -> &mut Self {
        hooks_mut(self).window_event.push(Box::new(hook));
        self
    }
}

pub fn claim_hooks_slot(app: &mut App) {
    hooks_mut(app);
}

pub fn install_scene_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    crate::renderer::install_scene_pass(renderer, pass);
}

pub fn install_swapchain_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    crate::renderer::install_swapchain_pass(renderer, pass);
}

pub fn run_install_hooks(hooks: &mut RenderHooks, renderer: &mut VulkanRenderer) {
    for hook in &mut hooks.install {
        hook(renderer);
    }
}

pub fn run_pre_render_hooks(
    hooks: &mut RenderHooks,
    renderer: &mut VulkanRenderer,
    storage: &mut Storage,
    resources: &mut ResourceMap,
) {
    for hook in &mut hooks.pre_render {
        hook(renderer, storage, resources);
    }
}

fn hooks_mut(app: &mut App) -> &mut RenderHooks {
    let slot = app.plugin_part();
    if slot.is_none() {
        *slot = Some(Box::new(RenderHooks::default()));
    }
    let ours = slot
        .as_ref()
        .map(|hooks| hooks.is::<RenderHooks>())
        .unwrap_or(false);
    assert!(
        ours,
        "the render hooks slot already holds another backend, so nightshade-vulkan and a second \
         render backend are both composed; compose one"
    );
    slot.as_mut()
        .and_then(|hooks| hooks.downcast_mut::<RenderHooks>())
        .expect("the render hooks slot holds the vulkan RenderHooks")
}

pub fn into_parts(app: App) -> (World, StageState, RenderHooks) {
    let mut app = app;
    let mut hooks = app
        .plugin_part()
        .take()
        .and_then(|hooks| hooks.downcast::<RenderHooks>().ok())
        .map(|hooks| *hooks)
        .unwrap_or_default();
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
