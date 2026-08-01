use crate::systems::lines::LinesPass;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::WgpuRenderer;
use nightshade_wgpu_geometry::backend::overlay_sub_pass_mut;

pub fn sync_debug_lines(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    let (graph, build) = renderer.graph_and_pass_build();
    let Some(lines_pass) = overlay_sub_pass_mut::<LinesPass>(graph) else {
        return;
    };
    lines_pass.upload_lines(build.device, build.queue, &inputs.frame.debug_lines.lines);
    match &inputs.frame.debug_lines.bounding_volumes {
        Some(bounding_volumes) => {
            lines_pass.upload_bounding_volumes(build.device, build.queue, bounding_volumes);
        }
        None => lines_pass.clear_bounding_volumes(),
    }
    match &inputs.frame.debug_lines.normals {
        Some(normals) => lines_pass.upload_normals(build.device, build.queue, normals),
        None => lines_pass.clear_normals(),
    }
}
