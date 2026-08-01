pub fn shader(device: &wgpu::Device, label: &str, source: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

pub fn uniform_at(binding: u32, seen_by: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn storage_at(
    binding: u32,
    writes: bool,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: !writes },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn texture_at(
    binding: u32,
    sample: wgpu::TextureSampleType,
    shape: wgpu::TextureViewDimension,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Texture {
            sample_type: sample,
            view_dimension: shape,
            multisampled: false,
        },
        count: None,
    }
}

pub fn painted_at(binding: u32) -> wgpu::BindGroupLayoutEntry {
    texture_at(
        binding,
        wgpu::TextureSampleType::Float { filterable: true },
        wgpu::TextureViewDimension::D2,
        wgpu::ShaderStages::FRAGMENT,
    )
}

pub fn depth_at(binding: u32, seen_by: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    texture_at(
        binding,
        wgpu::TextureSampleType::Depth,
        wgpu::TextureViewDimension::D2,
        seen_by,
    )
}

pub fn sampler_at(
    binding: u32,
    kind: wgpu::SamplerBindingType,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Sampler(kind),
        count: None,
    }
}

pub fn filtering_at(binding: u32) -> wgpu::BindGroupLayoutEntry {
    sampler_at(
        binding,
        wgpu::SamplerBindingType::Filtering,
        wgpu::ShaderStages::FRAGMENT,
    )
}

pub fn storage_texture_at(
    binding: u32,
    format: wgpu::TextureFormat,
    access: wgpu::StorageTextureAccess,
    shape: wgpu::TextureViewDimension,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::StorageTexture {
            access,
            format,
            view_dimension: shape,
        },
        count: None,
    }
}

pub fn bind_layout(
    device: &wgpu::Device,
    label: &str,
    entries: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries,
    })
}

pub fn binding(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    holds: Vec<wgpu::BindingResource>,
) -> wgpu::BindGroup {
    binding_at(
        device,
        label,
        layout,
        holds
            .into_iter()
            .enumerate()
            .map(|(slot, resource)| (slot as u32, resource))
            .collect(),
    )
}

pub fn binding_at(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    holds: Vec<(u32, wgpu::BindingResource)>,
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = holds
        .into_iter()
        .map(|(binding, resource)| wgpu::BindGroupEntry { binding, resource })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

pub fn pipeline_layout(
    device: &wgpu::Device,
    label: &str,
    layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::PipelineLayout {
    let held: Vec<Option<&wgpu::BindGroupLayout>> =
        layouts.iter().map(|layout| Some(*layout)).collect();
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &held,
        immediate_size: 0,
    })
}

pub fn uniform(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    buffer(
        device,
        label,
        size,
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    )
}

pub fn storage(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    buffer(
        device,
        label,
        size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    )
}

pub fn buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}

pub fn smooth(device: &wgpu::Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    })
}

pub fn blocky(device: &wgpu::Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

pub fn surface(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
    size: [u32; 3],
    mips: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    sampled(device, label, format, size, mips, usage, 1)
}

pub fn sampled(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
    size: [u32; 3],
    mips: u32,
    usage: wgpu::TextureUsages,
    samples: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: crate::renderer::texture_extent::texture_extent(device, size[0], size[1], size[2]),
        mip_level_count: mips,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub fn view_of(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::TextureView {
    surface(device, label, format, [width, height, 1], 1, usage)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn depth_state(format: wgpu::TextureFormat, writes: bool) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format,
        depth_write_enabled: Some(writes),
        depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

pub fn depth_nearer(format: wgpu::TextureFormat) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        depth_compare: Some(wgpu::CompareFunction::Greater),
        ..depth_state(format, true)
    }
}

pub fn depth_matched(format: wgpu::TextureFormat) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        depth_compare: Some(wgpu::CompareFunction::Equal),
        ..depth_state(format, false)
    }
}

pub fn depth_over(format: wgpu::TextureFormat) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format,
        depth_write_enabled: Some(false),
        depth_compare: Some(wgpu::CompareFunction::Always),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

pub fn sloped(format: wgpu::TextureFormat, constant: i32, slope: f32) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        bias: wgpu::DepthBiasState {
            constant,
            slope_scale: slope,
            clamp: 0.0,
        },
        ..depth_state(format, true)
    }
}

pub fn opaque(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

pub fn blended(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

pub fn additive(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

pub fn compute(
    device: &wgpu::Device,
    label: &str,
    module: &wgpu::ShaderModule,
    entry: &str,
    layout: &wgpu::PipelineLayout,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
    })
}

pub struct Pipe<'a> {
    device: &'a wgpu::Device,
    label: &'a str,
    shader: &'a wgpu::ShaderModule,
    layout: Option<&'a wgpu::PipelineLayout>,
    buffers: &'a [wgpu::VertexBufferLayout<'a>],
    targets: Vec<Option<wgpu::ColorTargetState>>,
    shades: bool,
    depth: Option<wgpu::DepthStencilState>,
    topology: wgpu::PrimitiveTopology,
    winding: wgpu::FrontFace,
    cull: Option<wgpu::Face>,
    vertex: &'a str,
    fragment: &'a str,
    samples: u32,
}

impl<'a> Pipe<'a> {
    pub fn new(device: &'a wgpu::Device, label: &'a str, shader: &'a wgpu::ShaderModule) -> Self {
        Self {
            device,
            label,
            shader,
            layout: None,
            buffers: &[],
            targets: Vec::new(),
            shades: false,
            depth: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
            winding: wgpu::FrontFace::Ccw,
            cull: None,
            vertex: "vertex_main",
            fragment: "fragment_main",
            samples: 1,
        }
    }

    pub fn layout(mut self, layout: &'a wgpu::PipelineLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn buffers(mut self, buffers: &'a [wgpu::VertexBufferLayout<'a>]) -> Self {
        self.buffers = buffers;
        self
    }

    pub fn target(mut self, target: wgpu::ColorTargetState) -> Self {
        self.targets = vec![Some(target)];
        self.shades = true;
        self
    }

    pub fn targets(mut self, targets: &[Option<wgpu::ColorTargetState>]) -> Self {
        self.targets = targets.to_vec();
        self.shades = true;
        self
    }

    pub fn depth(mut self, depth: wgpu::DepthStencilState) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn strip(mut self) -> Self {
        self.topology = wgpu::PrimitiveTopology::TriangleStrip;
        self
    }

    pub fn lines(mut self) -> Self {
        self.topology = wgpu::PrimitiveTopology::LineList;
        self
    }

    pub fn front_face(mut self, winding: wgpu::FrontFace) -> Self {
        self.winding = winding;
        self
    }

    pub fn cull(mut self) -> Self {
        self.cull = Some(wgpu::Face::Back);
        self
    }

    pub fn cull_front(mut self) -> Self {
        self.cull = Some(wgpu::Face::Front);
        self
    }

    pub fn vertex(mut self, entry: &'a str) -> Self {
        self.vertex = entry;
        self
    }

    pub fn fragment(mut self, entry: &'a str) -> Self {
        self.fragment = entry;
        self.shades = true;
        self
    }

    pub fn samples(mut self, samples: u32) -> Self {
        self.samples = samples;
        self
    }

    pub fn make(self) -> wgpu::RenderPipeline {
        let targets = self.targets.clone();
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.label),
                layout: self.layout,
                vertex: wgpu::VertexState {
                    module: self.shader,
                    entry_point: Some(self.vertex),
                    buffers: self.buffers,
                    compilation_options: Default::default(),
                },
                fragment: self.shades.then(|| wgpu::FragmentState {
                    module: self.shader,
                    entry_point: Some(self.fragment),
                    targets: &targets,
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: self.topology,
                    front_face: self.winding,
                    cull_mode: self.cull,
                    ..Default::default()
                },
                depth_stencil: self.depth.clone(),
                multisample: wgpu::MultisampleState {
                    count: self.samples,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            })
    }
}
