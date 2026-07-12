mod execute;
mod prepare;

use super::pass::MeshPass;
use crate::wgpu::render_configs::RenderInputs;
use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};

impl PassNode<RenderInputs> for MeshPass {
    fn name(&self) -> &str {
        "mesh_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["shadow_depth", "spotlight_shadow_atlas"]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth", "entity_id", "view_normals", "velocity"]
    }

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        self.prepare_pass_node(device, queue, configs);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        self.execute_pass_node(context)
    }
}
