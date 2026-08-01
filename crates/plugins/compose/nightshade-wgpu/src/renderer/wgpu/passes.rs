use crate::renderer::wgpu::build;
pub mod postprocess;
pub use postprocess::{
    BlitPass, BloomPass, DepthOfFieldPass, FxaaPass, PostProcessPass, TaaPass, ViewportComposePass,
};

pub fn grow_storage_buffer<T>(
    device: &wgpu::Device,
    buffer: &mut wgpu::Buffer,
    capacity: &mut usize,
    required: usize,
    usage: wgpu::BufferUsages,
    label: &str,
) -> bool {
    if required <= *capacity {
        return false;
    }
    let new_capacity = (required as f32 * 2.0).ceil() as usize;
    *buffer = build::buffer(
        device,
        label,
        (std::mem::size_of::<T>() * new_capacity) as u64,
        usage,
    );
    *capacity = new_capacity;
    true
}

pub fn sampled_texture_uniform_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    build::bind_layout(
        device,
        label,
        &[
            build::painted_at(0),
            build::filtering_at(1),
            build::uniform_at(2, wgpu::ShaderStages::FRAGMENT),
        ],
    )
}

pub fn fullscreen_pipeline(
    device: &wgpu::Device,
    label: &str,
    pipeline_layout: &wgpu::PipelineLayout,
    shader_module: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    build::Pipe::new(device, label, shader_module)
        .layout(pipeline_layout)
        .target(build::opaque(format))
        .make()
}
