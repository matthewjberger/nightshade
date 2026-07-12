use super::passes;
use crate::wgpu::rendergraph::{
    render_graph_get_pass_mut, render_graph_resize_all_transient_proportional,
};

impl super::WgpuRenderer {
    pub fn cleanup_unused_camera_viewports(
        &mut self,
        active_camera_entities: &std::collections::HashSet<freecs::Entity>,
    ) {
        self.camera_viewports
            .retain(|entity, _| active_camera_entities.contains(entity));
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

        let width = width.max(1);
        let height = height.max(1);

        let (old_width, old_height) = self.render_buffer_size;
        let _ = render_graph_resize_all_transient_proportional(
            &mut self.graph,
            old_width,
            old_height,
            width,
            height,
        );

        if let Some(mesh_pass) = render_graph_get_pass_mut(&mut self.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            mesh_pass.resize(&self.device, width, height);
        }

        if let Some(bloom_pass) = render_graph_get_pass_mut(&mut self.graph, "bloom_pass")
            && let Some(bloom_pass) =
                (bloom_pass as &mut dyn std::any::Any).downcast_mut::<passes::BloomPass>()
        {
            bloom_pass.resize(&self.device, &self.queue, width, height);
        }

        #[cfg(feature = "debug_render")]
        if let Some(outline_pass) = render_graph_get_pass_mut(&mut self.graph, "outline_pass")
            && let Some(outline_pass) =
                (outline_pass as &mut dyn std::any::Any).downcast_mut::<passes::OutlinePass>()
        {
            outline_pass.resize(width, height);
        }

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

    pub fn cleanup_world(&mut self, world_id: u64) {
        if let Some(mesh_pass) = render_graph_get_pass_mut(&mut self.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            mesh_pass.cleanup_world_state(world_id);
        }
    }

    pub fn cleanup_stale_worlds(&mut self, max_age_frames: u64) {
        if let Some(mesh_pass) = render_graph_get_pass_mut(&mut self.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            mesh_pass.cleanup_stale_world_states(max_age_frames);
        }
    }
}
