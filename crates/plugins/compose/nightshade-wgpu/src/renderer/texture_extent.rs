pub fn texture_extent(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    layers: u32,
) -> wgpu::Extent3d {
    let limits = device.limits();
    wgpu::Extent3d {
        width: width.clamp(1, limits.max_texture_dimension_2d),
        height: height.clamp(1, limits.max_texture_dimension_2d),
        depth_or_array_layers: layers.clamp(1, limits.max_texture_array_layers),
    }
}
