use crate::renderer::wgpu::DEPTH_PICK_SAMPLE_SIZE;
use crate::renderer::wgpu::WgpuRenderer;
use crate::renderer::wgpu::build;
use crate::renderer::wgpu::rendergraph::render_graph_get_texture_view;

pub struct DepthPickReadback {
    pub depth_values: Vec<f32>,
    pub entity_id_values: Vec<u32>,
    pub sample_size: u32,
    pub center: (u32, u32),
    pub texture_size: (u32, u32),
    pub camera: Option<nightshade::ecs::Entity>,
}

pub fn poll_depth_pick(renderer: &mut WgpuRenderer) -> Option<DepthPickReadback> {
    if !renderer.depth_pick.pending {
        return None;
    }
    let _ = renderer.device.poll(wgpu::PollType::Poll);
    if !renderer
        .depth_pick
        .map_complete
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return None;
    }

    let buffer_slice = renderer.depth_pick.staging_buffer.slice(..);
    let data = buffer_slice.get_mapped_range();
    let mut depth_values = Vec::new();
    let mut entity_id_values = Vec::new();
    for chunk in data.as_chunks::<8>().0 {
        depth_values.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        entity_id_values.push(u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]));
    }
    drop(data);
    renderer.depth_pick.staging_buffer.unmap();

    let readback = DepthPickReadback {
        depth_values,
        entity_id_values,
        sample_size: DEPTH_PICK_SAMPLE_SIZE,
        center: renderer.depth_pick.center,
        texture_size: renderer.depth_pick.texture_size,
        camera: renderer.depth_pick.camera,
    };

    renderer.depth_pick.pending = false;
    renderer
        .depth_pick
        .map_complete
        .store(false, std::sync::atomic::Ordering::Relaxed);

    Some(readback)
}

pub fn dispatch_pick_compute(
    renderer: &mut WgpuRenderer,
    screen_x: u32,
    screen_y: u32,
    camera: Option<nightshade::ecs::Entity>,
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

    let bind_group = build::binding(
        &renderer.device,
        "Depth Pick Bind Group",
        &renderer.depth_pick.bind_group_layout,
        vec![
            wgpu::BindingResource::TextureView(depth_texture_view),
            renderer.depth_pick.storage_buffer.as_entire_binding(),
            renderer.depth_pick.uniform_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(entity_id_texture_view),
        ],
    );

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
