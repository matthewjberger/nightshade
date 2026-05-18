//! Application state trait and render-graph resource handles.
//!
//! Defines the [`State`] trait that user applications implement, along with
//! [`RenderResources`] which is passed to [`State::configure_render_graph`].

use crate::ecs::world::World;

/// Resource IDs for render graph configuration.
///
/// Provided to [`State::configure_render_graph`] for custom pass setup.
pub struct RenderResources {
    /// Main HDR scene color target.
    pub scene_color: crate::render::wgpu::rendergraph::ResourceId,
    /// Depth buffer (reverse-Z format).
    pub depth: crate::render::wgpu::rendergraph::ResourceId,
    /// Compute shader output texture.
    pub compute_output: crate::render::wgpu::rendergraph::ResourceId,
    /// Final swapchain output.
    pub swapchain: crate::render::wgpu::rendergraph::ResourceId,
    /// View-space normals for SSAO.
    pub view_normals: crate::render::wgpu::rendergraph::ResourceId,
    /// Raw SSAO output before blur.
    pub ssao_raw: crate::render::wgpu::rendergraph::ResourceId,
    /// Blurred SSAO output.
    pub ssao: crate::render::wgpu::rendergraph::ResourceId,
    /// Raw SSGI output before blur.
    pub ssgi_raw: crate::render::wgpu::rendergraph::ResourceId,
    /// Blurred SSGI output.
    pub ssgi: crate::render::wgpu::rendergraph::ResourceId,
    /// Raw SSR output before blur.
    pub ssr_raw: crate::render::wgpu::rendergraph::ResourceId,
    /// Blurred SSR output.
    pub ssr: crate::render::wgpu::rendergraph::ResourceId,
    pub surface_width: u32,
    pub surface_height: u32,
}

/// Builder closure that constructs the next [`State`] when the engine
/// transitions. Stored on the `Window` resource as
/// `world.resources.window.next_state` and invoked at the end of the frame.
pub type NextStateBuilder = Box<dyn FnOnce(&mut World) -> Box<dyn State>>;

/// Application state trait for game logic and rendering.
///
/// Implement this trait to define your application behavior. The engine calls
/// these methods at appropriate points in the frame loop.
pub trait State {
    /// Called once after the renderer is created to set up the initial scene.
    fn initialize(&mut self, _world: &mut World) {}
    fn configure_render_graph(
        &mut self,
        _: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        _: &wgpu::Device,
        _: wgpu::TextureFormat,
        _: RenderResources,
    ) {
    }
    /// Called every frame to run game logic.
    fn run_systems(&mut self, _world: &mut World) {}
    /// Called before main rendering to allow custom world rendering.
    fn pre_render(
        &mut self,
        _renderer: &mut crate::render::wgpu::WgpuRenderer,
        _world: &mut World,
    ) {
    }
    /// Update render graph state each frame.
    fn update_render_graph(
        &mut self,
        _graph: &mut crate::render::wgpu::rendergraph::RenderGraph<World>,
        _world: &World,
    ) {
    }
}
