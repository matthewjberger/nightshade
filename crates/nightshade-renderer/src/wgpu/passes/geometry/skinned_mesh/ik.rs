use crate::config::SkinnedAnimationSnapshot;
use crate::skinning::SkinningCache;
use nightshade_ecs::Entity;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuIkChain {
    root_index: u32,
    mid_index: u32,
    tip_index: u32,
    has_pole: u32,
    target: [f32; 4],
    pole: [f32; 4],
    weight: f32,
    descendant_start: u32,
    descendant_count: u32,
    pad0: u32,
}

struct IkChainLayout {
    player: Entity,
    chain_index: usize,
    root_index: u32,
    mid_index: u32,
    tip_index: u32,
    has_pole: u32,
    descendant_start: u32,
    descendant_count: u32,
}

pub(super) struct SkinnedIkGpu {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    chains_buffer: wgpu::Buffer,
    tip_descendants_buffer: wgpu::Buffer,
    layouts: Vec<IkChainLayout>,
    cached_static_signature: u64,
    bone_transforms_generation: u64,
}

fn sized_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn init_buffer(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
    let fallback = [0u8; 16];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: if data.is_empty() { &fallback } else { data },
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

impl SkinnedIkGpu {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "ik_compute.wgsl",
            include_str!("../../../shaders/ik_compute.wgsl"),
        );

        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..3)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding != 0,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Skinned IK Bind Group Layout"),
            entries: &entries,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skinned IK Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Skinned IK Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            chains_buffer: sized_buffer(device, "IK Chains", 16),
            tip_descendants_buffer: sized_buffer(device, "IK Tip Descendants", 16),
            layouts: Vec::new(),
            cached_static_signature: u64::MAX,
            bone_transforms_generation: u64::MAX,
        }
    }

    pub(super) fn chain_count(&self) -> u32 {
        self.layouts.len() as u32
    }

    pub(super) fn dispatch(&self, compute_pass: &mut wgpu::ComputePass) {
        if self.layouts.is_empty() {
            return;
        }
        if let Some(bind_group) = self.bind_group.as_ref() {
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups((self.layouts.len() as u32).div_ceil(64), 1, 1);
        }
    }

    pub(super) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
        skinning_cache: &SkinningCache,
        bone_transforms_buffer: &wgpu::Buffer,
        bone_transforms_generation: u64,
    ) {
        let snapshot = &configs.scene.render_animation;

        if snapshot.signature != self.cached_static_signature
            || self.bone_transforms_generation != bone_transforms_generation
        {
            self.rebuild_static(device, snapshot, skinning_cache);
            self.cached_static_signature = snapshot.signature;
            self.bone_transforms_generation = bone_transforms_generation;
            self.bind_group = Some(self.create_bind_group(device, bone_transforms_buffer));
        }

        self.upload_chains(queue, snapshot);
    }

    fn rebuild_static(
        &mut self,
        device: &wgpu::Device,
        snapshot: &SkinnedAnimationSnapshot,
        skinning_cache: &SkinningCache,
    ) {
        self.layouts.clear();
        let mut tip_descendants: Vec<u32> = Vec::new();

        for skeleton in &snapshot.skeletons {
            if skeleton.ik_chains.is_empty() {
                continue;
            }
            let Some(&skin_index) = skinning_cache
                .entity_skin_indices
                .get(&skeleton.skin_entity)
            else {
                continue;
            };
            let base_bone_index = skinning_cache.get_base_bone_index(skin_index);

            let mut parent_of: std::collections::HashMap<u32, Option<u32>> =
                std::collections::HashMap::new();
            for joint in &skeleton.joints_ordered {
                parent_of.insert(joint.local_index, joint.parent_local);
            }

            for chain in &skeleton.ik_chains {
                let descendant_start = tip_descendants.len() as u32;
                for joint in &skeleton.joints_ordered {
                    if joint.local_index == chain.tip_local {
                        continue;
                    }
                    let mut current = joint.parent_local;
                    let mut passes_tip = false;
                    while let Some(parent) = current {
                        if parent == chain.tip_local {
                            passes_tip = true;
                            break;
                        }
                        current = parent_of.get(&parent).copied().flatten();
                    }
                    if passes_tip {
                        tip_descendants.push(base_bone_index + joint.local_index);
                    }
                }
                let descendant_count = tip_descendants.len() as u32 - descendant_start;

                self.layouts.push(IkChainLayout {
                    player: skeleton.player_entity,
                    chain_index: chain.chain_index as usize,
                    root_index: base_bone_index + chain.root_local,
                    mid_index: base_bone_index + chain.mid_local,
                    tip_index: base_bone_index + chain.tip_local,
                    has_pole: u32::from(chain.has_pole),
                    descendant_start,
                    descendant_count,
                });
            }
        }

        self.tip_descendants_buffer = init_buffer(
            device,
            "IK Tip Descendants",
            bytemuck::cast_slice(&tip_descendants),
        );
        self.chains_buffer = sized_buffer(
            device,
            "IK Chains",
            (self.layouts.len().max(1) * std::mem::size_of::<GpuIkChain>()) as u64,
        );
    }

    fn upload_chains(&self, queue: &wgpu::Queue, snapshot: &SkinnedAnimationSnapshot) {
        if self.layouts.is_empty() {
            return;
        }
        let chains: Vec<GpuIkChain> = self
            .layouts
            .iter()
            .map(|layout| {
                let runtime = snapshot
                    .ik_runtime
                    .get(&(layout.player, layout.chain_index))
                    .copied()
                    .unwrap_or([0.0; 8]);
                let weight = if runtime[7] > 0.0 { runtime[6] } else { 0.0 };
                GpuIkChain {
                    root_index: layout.root_index,
                    mid_index: layout.mid_index,
                    tip_index: layout.tip_index,
                    has_pole: layout.has_pole,
                    target: [runtime[0], runtime[1], runtime[2], 0.0],
                    pole: [runtime[3], runtime[4], runtime[5], 0.0],
                    weight,
                    descendant_start: layout.descendant_start,
                    descendant_count: layout.descendant_count,
                    pad0: 0,
                }
            })
            .collect();
        queue.write_buffer(&self.chains_buffer, 0, bytemuck::cast_slice(&chains));
    }

    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        bone_transforms_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinned IK Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bone_transforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.chains_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.tip_descendants_buffer.as_entire_binding(),
                },
            ],
        })
    }
}
