use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct HizParams {
    src_mip_size: [u32; 2],
    dst_mip_size: [u32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpdParams {
    src_size: [u32; 2],
    mip_count: u32,
    padding: u32,
}

struct MipLevelData {
    params_buffer: wgpu::Buffer,
    workgroup_x: u32,
    workgroup_y: u32,
}

const SPD_MIP_COUNT: u32 = 4;

pub struct HizPass {
    downsample_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,

    spd_pipeline: wgpu::ComputePipeline,
    spd_bind_group_layout: wgpu::BindGroupLayout,
    spd_params_buffer: wgpu::Buffer,
    spd_bind_group: Option<wgpu::BindGroup>,

    hiz_texture: Option<wgpu::Texture>,
    hiz_full_view: Option<wgpu::TextureView>,
    mip_views: Vec<wgpu::TextureView>,
    mip_storage_views: Vec<wgpu::TextureView>,
    mip_level_data: Vec<MipLevelData>,
    bind_groups: Vec<wgpu::BindGroup>,

    dummy_view: wgpu::TextureView,

    width: u32,
    height: u32,
    mip_count: u32,
    bind_groups_dirty: bool,
}

impl HizPass {
    pub fn new(build: PassBuild) -> Self {
        let PassBuild {
            device,
            queue,
            composer,
            ..
        } = build;
        let shader = composer.compile_wgsl(
            device,
            "Hi-Z Downsample Shader",
            include_str!("../shaders/hiz_downsample.wgsl"),
        );

        let bind_group_layout = build::bind_layout(
            device,
            "Hi-Z Downsample Bind Group Layout",
            &[
                build::texture_at(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_texture_at(
                    1,
                    wgpu::TextureFormat::R32Float,
                    wgpu::StorageTextureAccess::WriteOnly,
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "Hi-Z Pipeline Layout", &[&bind_group_layout]);

        let downsample_pipeline = build::compute(
            device,
            "Hi-Z Downsample Pipeline",
            &shader,
            "main",
            &pipeline_layout,
        );

        let spd_shader = composer.compile_wgsl(
            device,
            "Hi-Z SPD Shader",
            include_str!("../shaders/hiz_spd.wgsl"),
        );

        let spd_bind_group_layout = build::bind_layout(
            device,
            "Hi-Z SPD Bind Group Layout",
            &[
                build::depth_at(0, wgpu::ShaderStages::COMPUTE),
                build::storage_texture_at(
                    1,
                    wgpu::TextureFormat::R32Float,
                    wgpu::StorageTextureAccess::WriteOnly,
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_texture_at(
                    2,
                    wgpu::TextureFormat::R32Float,
                    wgpu::StorageTextureAccess::WriteOnly,
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_texture_at(
                    3,
                    wgpu::TextureFormat::R32Float,
                    wgpu::StorageTextureAccess::WriteOnly,
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_texture_at(
                    4,
                    wgpu::TextureFormat::R32Float,
                    wgpu::StorageTextureAccess::WriteOnly,
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::uniform_at(5, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let spd_pipeline_layout = build::pipeline_layout(
            device,
            "Hi-Z SPD Pipeline Layout",
            &[&spd_bind_group_layout],
        );

        let spd_pipeline = build::compute(
            device,
            "Hi-Z SPD Pipeline",
            &spd_shader,
            "main",
            &spd_pipeline_layout,
        );

        let spd_params_buffer = build::uniform(
            device,
            "Hi-Z SPD Params Buffer",
            std::mem::size_of::<SpdParams>() as u64,
        );

        let dummy_texture = nightshade_wgpu::backend::placeholder::placeholder_texture(
            device,
            queue,
            "Hi-Z Dummy Texture",
            wgpu::TextureFormat::R32Float,
            1,
            &nightshade_wgpu::backend::placeholder::NEAR_NOTHING_R32,
        );
        let dummy_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            downsample_pipeline,
            bind_group_layout,
            spd_pipeline,
            spd_bind_group_layout,
            spd_params_buffer,
            spd_bind_group: None,
            hiz_texture: None,
            hiz_full_view: None,
            mip_views: Vec::new(),
            mip_storage_views: Vec::new(),
            mip_level_data: Vec::new(),
            bind_groups: Vec::new(),
            dummy_view,
            width: 0,
            height: 0,
            mip_count: 0,
            bind_groups_dirty: true,
        }
    }

    pub fn invalidate_bind_groups(&mut self) {
        self.bind_groups_dirty = true;
    }

    fn calculate_mip_count(width: u32, height: u32) -> u32 {
        let max_dim = width.max(height);
        (max_dim as f32).log2().floor() as u32 + 1
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        self.bind_groups.clear();
        self.spd_bind_group = None;
        self.bind_groups_dirty = true;
        self.mip_count = Self::calculate_mip_count(width, height);

        let hiz_texture = build::surface(
            device,
            "Hi-Z Texture",
            wgpu::TextureFormat::R32Float,
            [width, height, 1],
            self.mip_count,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST,
        );

        self.hiz_full_view = Some(hiz_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Hi-Z Full View"),
            format: Some(wgpu::TextureFormat::R32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(self.mip_count),
            base_array_layer: 0,
            array_layer_count: None,
            ..Default::default()
        }));

        self.mip_views.clear();
        self.mip_storage_views.clear();

        for mip in 0..self.mip_count {
            let view = hiz_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Hi-Z Mip {} View", mip)),
                format: Some(wgpu::TextureFormat::R32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
                ..Default::default()
            });
            self.mip_views.push(view);

            let storage_view = hiz_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Hi-Z Mip {} Storage View", mip)),
                format: Some(wgpu::TextureFormat::R32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
                ..Default::default()
            });
            self.mip_storage_views.push(storage_view);
        }

        self.hiz_texture = Some(hiz_texture);

        self.mip_level_data.clear();
        self.bind_groups.clear();

        for mip in SPD_MIP_COUNT..self.mip_count {
            let src_width = (width >> (mip - 1)).max(1);
            let src_height = (height >> (mip - 1)).max(1);
            let dst_width = (width >> mip).max(1);
            let dst_height = (height >> mip).max(1);

            let params = HizParams {
                src_mip_size: [src_width, src_height],
                dst_mip_size: [dst_width, dst_height],
            };
            let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Hi-Z Mip {} Params Buffer", mip)),
                size: std::mem::size_of::<HizParams>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            params_buffer
                .slice(..)
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::bytes_of(&params));
            params_buffer.unmap();

            self.mip_level_data.push(MipLevelData {
                params_buffer,
                workgroup_x: dst_width.div_ceil(8),
                workgroup_y: dst_height.div_ceil(8),
            });
        }

        let spd_params = SpdParams {
            src_size: [width, height],
            mip_count: self.mip_count.min(SPD_MIP_COUNT),
            padding: 0,
        };
        self.spd_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hi-Z SPD Params Buffer"),
            size: std::mem::size_of::<SpdParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        self.spd_params_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::bytes_of(&spd_params));
        self.spd_params_buffer.unmap();
    }

    pub fn rebuild_bind_groups(&mut self, device: &wgpu::Device, depth_view: &wgpu::TextureView) {
        if !self.bind_groups_dirty {
            return;
        }

        self.bind_groups.clear();
        self.spd_bind_group = None;

        if self.mip_count == 0 {
            self.bind_groups_dirty = false;
            return;
        }

        let spd_mips_to_process = self.mip_count.min(SPD_MIP_COUNT) as usize;
        if spd_mips_to_process >= 1 && self.mip_storage_views.len() >= spd_mips_to_process {
            let mip_view_or_dummy = |index: usize| -> &wgpu::TextureView {
                if index < self.mip_storage_views.len() {
                    &self.mip_storage_views[index]
                } else {
                    &self.mip_storage_views[0]
                }
            };

            self.spd_bind_group = Some(build::binding(
                device,
                "Hi-Z SPD Bind Group",
                &self.spd_bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(depth_view),
                    wgpu::BindingResource::TextureView(mip_view_or_dummy(0)),
                    wgpu::BindingResource::TextureView(mip_view_or_dummy(1)),
                    wgpu::BindingResource::TextureView(mip_view_or_dummy(2)),
                    wgpu::BindingResource::TextureView(mip_view_or_dummy(3)),
                    self.spd_params_buffer.as_entire_binding(),
                ],
            ));
        }

        for mip in SPD_MIP_COUNT as usize..self.mip_count as usize {
            let bind_group = build::binding(
                device,
                &format!("Hi-Z Mip {} Bind Group", mip),
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(&self.mip_views[mip - 1]),
                    wgpu::BindingResource::TextureView(&self.mip_storage_views[mip]),
                    self.mip_level_data[mip - SPD_MIP_COUNT as usize]
                        .params_buffer
                        .as_entire_binding(),
                ],
            );
            self.bind_groups.push(bind_group);
        }

        self.bind_groups_dirty = false;
    }

    pub fn execute(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        if let Some(spd_bind_group) = &self.spd_bind_group {
            let workgroups_x = self.width.div_ceil(64);
            let workgroups_y = self.height.div_ceil(64);

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Hi-Z SPD Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.spd_pipeline);
            compute_pass.set_bind_group(0, spd_bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        for (bind_group_index, bind_group) in self.bind_groups.iter().enumerate() {
            let mip_data = &self.mip_level_data[bind_group_index];

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Hi-Z Downsample Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.downsample_pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(mip_data.workgroup_x, mip_data.workgroup_y, 1);
        }
    }

    pub fn hiz_view(&self) -> Option<&wgpu::TextureView> {
        self.hiz_full_view.as_ref()
    }

    pub fn hiz_view_or_dummy(&self) -> &wgpu::TextureView {
        self.hiz_full_view.as_ref().unwrap_or(&self.dummy_view)
    }

    pub fn mip_count(&self) -> u32 {
        self.mip_count
    }
}
