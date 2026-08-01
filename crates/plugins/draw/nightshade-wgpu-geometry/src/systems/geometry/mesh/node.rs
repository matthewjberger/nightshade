type Mapping = (
    nightshade_render::data::asset_id::TextureId,
    nightshade_wgpu::backend::MaterialTextureLayer,
);
use nightshade_wgpu::wgpu;
mod execute;
mod prepare;

use super::pass::MeshPass;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition};

const MESH_PASS_DECL: PassDecl = PassDecl {
    name: "mesh_pass",
    position: PassPosition::Geometry,
    reads: &["shadow_depth", "spotlight_shadow_atlas"],
    optional_reads: &[
        "point_shadow_cubemap",
        "transmission_grab",
        "brdf_lut",
        "ibl_irradiance",
        "ibl_prefiltered",
        "ibl_irradiance_b",
        "ibl_prefiltered_b",
    ],
    reads_writes: &["color", "depth", "entity_id", "view_normals", "velocity"],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for MeshPass {
    fn decl(&self) -> &PassDecl {
        &MESH_PASS_DECL
    }

    fn publish(&mut self, publisher: &mut nightshade_wgpu::backend::ResourcePublisher<'_>) {
        if !self.transmission.publish_pending {
            return;
        }
        self.transmission.publish_pending = false;
        publisher.texture(
            "transmission_grab",
            None,
            self.transmission.color_view.clone(),
            self.transmission.size.0,
            self.transmission.size.1,
        );
    }

    fn material_update(
        &mut self,
        device: &wgpu::Device,
        update: &nightshade_wgpu::backend::MaterialUpdate<'_>,
    ) {
        match update {
            nightshade_wgpu::backend::MaterialUpdate::Register {
                name,
                view,
                sampler,
                mapping,
            } => {
                self.register_texture_with_data(
                    (*name).to_string(),
                    (*view).clone(),
                    (*sampler).clone(),
                );
                if let Some(pair) = mapping.and_then(|value| value.downcast_ref::<Mapping>()) {
                    self.add_material_layer_mapping(pair.0, pair.1);
                }
            }
            nightshade_wgpu::backend::MaterialUpdate::Unregister { name } => {
                self.unregister_texture(name);
            }
            nightshade_wgpu::backend::MaterialUpdate::Mapping { mapping } => {
                if let Some(pair) = mapping.downcast_ref::<Mapping>() {
                    self.add_material_layer_mapping(pair.0, pair.1);
                }
            }
            nightshade_wgpu::backend::MaterialUpdate::Rebind { arrays } => {
                if let Some(arrays) =
                    arrays.downcast_ref::<nightshade_wgpu::backend::MaterialTextureArrays>()
                {
                    self.apply_material_textures(device, arrays);
                }
            }
        }
    }

    fn resize(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, width: u32, height: u32) {
        MeshPass::resize(self, device, width, height);
    }

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        self.take_frame_dirty_state(configs);
        self.prepare_pass_node(device, queue, configs);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        self.execute_pass_node(context)
    }
}
