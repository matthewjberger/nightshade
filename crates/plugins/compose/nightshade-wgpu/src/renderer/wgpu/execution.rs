use super::passes;
use crate::renderer::wgpu::rendergraph::{
    render_graph_get_pass_mut, render_graph_resize_all_transient_proportional,
};

impl super::WgpuRenderer {
    pub(super) fn clamp_render_extent(&self, width: u32, height: u32) -> (u32, u32) {
        let extent =
            crate::renderer::texture_extent::texture_extent(&self.device, width, height, 1);
        (extent.width, extent.height)
    }

    pub fn cleanup_unused_camera_viewports(
        &mut self,
        active_camera_entities: &std::collections::HashSet<nightshade::ecs::Entity>,
    ) {
        self.camera_viewports
            .retain(|entity, _| active_camera_entities.contains(entity));

        if let Some(taa_pass) = render_graph_get_pass_mut(&mut self.graph, "taa_pass")
            && let Some(taa_pass) =
                (taa_pass as &mut dyn std::any::Any).downcast_mut::<passes::TaaPass>()
        {
            taa_pass.retain_cameras(active_camera_entities);
        }
    }

    pub fn resize_render_buffers_for_window(&mut self, width: u32, height: u32) {
        let already_dispatched_at_this_size = self
            .window_render_state
            .recent_buffer_sizes
            .contains(&(width, height));
        if already_dispatched_at_this_size && self.render_buffer_size == (width, height) {
            return;
        }
        if !already_dispatched_at_this_size {
            self.window_render_state
                .recent_buffer_sizes
                .push((width, height));
            if self.window_render_state.recent_buffer_sizes.len()
                > super::RECENT_BUFFER_SIZE_CAPACITY
            {
                self.window_render_state.recent_buffer_sizes.remove(0);
            }
        }
        self.resize_render_buffers(width, height);
    }

    pub fn record_window_dispatch(&mut self, settings_version: u64) {
        self.window_render_state.last_settings_version = Some(settings_version);
    }

    pub fn resize_render_buffers(&mut self, width: u32, height: u32) {
        if self.render_buffer_size == (width, height) {
            return;
        }

        let (width, height) = self.clamp_render_extent(width, height);
        if self.render_buffer_size == (width, height) {
            return;
        }

        let (old_width, old_height) = self.render_buffer_size;
        let _ = render_graph_resize_all_transient_proportional(
            &mut self.graph,
            old_width,
            old_height,
            width,
            height,
        );

        crate::renderer::wgpu::rendergraph::render_graph_resize_passes(
            &mut self.graph,
            &self.device,
            &self.queue,
            width,
            height,
        );

        self.render_buffer_size = (width, height);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }
}
