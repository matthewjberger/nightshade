//! GPU depth-pick dispatch: samples the depth and entity-id targets around a
//! screen position into a staging buffer the caller reads back on a later
//! frame once the async map completes.

use crate::wgpu::DEPTH_PICK_SAMPLE_SIZE;
use crate::wgpu::WgpuRenderer;
use crate::wgpu::rendergraph::render_graph_get_texture_view;

/// Dispatches the depth-pick compute shader around a screen position, starting an async readback the caller polls on a later frame.
///
/// No-op while a previous pick is still pending.
pub fn dispatch_pick_compute(
    renderer: &mut WgpuRenderer,
    screen_x: u32,
    screen_y: u32,
    camera: Option<nightshade_ecs::Entity>,
) {
    if renderer.depth_pick.pending {
        return;
    }
    let Some(depth_texture_view) =
        render_graph_get_texture_view(&renderer.graph, renderer.targets.depth)
    else {
        return;
    };
    let Some(entity_id_texture_view) =
        render_graph_get_texture_view(&renderer.graph, renderer.targets.entity_id)
    else {
        return;
    };

    let uniform_data: [u32; 4] = [screen_x, screen_y, DEPTH_PICK_SAMPLE_SIZE, 0];
    renderer.queue.write_buffer(
        &renderer.depth_pick.uniform_buffer,
        0,
        bytemuck::cast_slice(&uniform_data),
    );

    let bind_group = renderer
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Depth Pick Bind Group"),
            layout: &renderer.depth_pick.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer.depth_pick.storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: renderer.depth_pick.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(entity_id_texture_view),
                },
            ],
        });

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Depth Pick Encoder"),
        });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Depth Pick Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&renderer.depth_pick.compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(DEPTH_PICK_SAMPLE_SIZE, DEPTH_PICK_SAMPLE_SIZE, 1);
    }

    encoder.copy_buffer_to_buffer(
        &renderer.depth_pick.storage_buffer,
        0,
        &renderer.depth_pick.staging_buffer,
        0,
        (DEPTH_PICK_SAMPLE_SIZE * DEPTH_PICK_SAMPLE_SIZE * 8) as u64,
    );

    renderer.queue.submit(std::iter::once(encoder.finish()));
    renderer.depth_pick.bind_group = Some(bind_group);
    renderer.depth_pick.pending = true;
    renderer.depth_pick.center = (screen_x, screen_y);
    renderer.depth_pick.texture_size = renderer.render_buffer_size;
    renderer.depth_pick.camera = camera;

    let map_complete = renderer.depth_pick.map_complete.clone();
    renderer
        .depth_pick
        .staging_buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |_| {
            map_complete.store(true, std::sync::atomic::Ordering::Relaxed);
        });
}
