use crate::config::SkinnedAnimationSnapshot;
use crate::skinning::SkinningCache;
use nightshade_ecs::Entity;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSpringChain {
    joint_start: u32,
    joint_count: u32,
    descendant_start: u32,
    descendant_count: u32,
    state_start: u32,
    reset: u32,
    enabled: u32,
    pad0: u32,
    gravity: [f32; 4],
    stiffness: f32,
    damping: f32,
    dt: f32,
    pad1: f32,
}

struct SpringChainLayout {
    player: Entity,
    chain_index: usize,
    joint_start: u32,
    joint_count: u32,
    descendant_start: u32,
    descendant_count: u32,
    state_start: u32,
}

pub(super) struct SkinnedSpringGpu {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    chains_buffer: wgpu::Buffer,
    joint_indices_buffer: wgpu::Buffer,
    tip_descendants_buffer: wgpu::Buffer,
    rest_axes_buffer: wgpu::Buffer,
    bone_lengths_buffer: wgpu::Buffer,
    state_buffer: wgpu::Buffer,
    layouts: Vec<SpringChainLayout>,
    reset_pending: bool,
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

impl SkinnedSpringGpu {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "spring_compute.wgsl",
            include_str!("../../../shaders/spring_compute.wgsl"),
        );

        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..7)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding != 0 && binding != 6,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Skinned Spring Bind Group Layout"),
            entries: &entries,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skinned Spring Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Skinned Spring Pipeline"),
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
            chains_buffer: sized_buffer(device, "Spring Chains", 16),
            joint_indices_buffer: sized_buffer(device, "Spring Joint Indices", 16),
            tip_descendants_buffer: sized_buffer(device, "Spring Tip Descendants", 16),
            rest_axes_buffer: sized_buffer(device, "Spring Rest Axes", 16),
            bone_lengths_buffer: sized_buffer(device, "Spring Bone Lengths", 16),
            state_buffer: sized_buffer(device, "Spring State", 32),
            layouts: Vec::new(),
            reset_pending: true,
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
        let mut joint_indices: Vec<u32> = Vec::new();
        let mut tip_descendants: Vec<u32> = Vec::new();
        let mut rest_axes: Vec<[f32; 4]> = Vec::new();
        let mut bone_lengths: Vec<f32> = Vec::new();
        let mut total_slots: u32 = 0;

        for skeleton in &snapshot.skeletons {
            if skeleton.spring_chains.is_empty() {
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

            for chain in &skeleton.spring_chains {
                if chain.joint_locals.len() < 2 {
                    continue;
                }
                let joint_start = joint_indices.len() as u32;
                for (index, &local) in chain.joint_locals.iter().enumerate() {
                    joint_indices.push(base_bone_index + local);
                    let axis = chain
                        .rest_axes
                        .get(index)
                        .copied()
                        .unwrap_or([0.0, 1.0, 0.0]);
                    rest_axes.push([axis[0], axis[1], axis[2], 0.0]);
                    bone_lengths.push(chain.bone_lengths.get(index).copied().unwrap_or(0.0));
                }
                let joint_count = chain.joint_locals.len() as u32;

                let tip_local = *chain.joint_locals.last().unwrap();
                let descendant_start = tip_descendants.len() as u32;
                for joint in &skeleton.joints_ordered {
                    if joint.local_index == tip_local {
                        continue;
                    }
                    let mut current = joint.parent_local;
                    let mut passes_tip = false;
                    while let Some(parent) = current {
                        if parent == tip_local {
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

                self.layouts.push(SpringChainLayout {
                    player: skeleton.player_entity,
                    chain_index: chain.chain_index as usize,
                    joint_start,
                    joint_count,
                    descendant_start,
                    descendant_count,
                    state_start: total_slots,
                });
                total_slots += joint_count;
            }
        }

        self.joint_indices_buffer = init_buffer(
            device,
            "Spring Joint Indices",
            bytemuck::cast_slice(&joint_indices),
        );
        self.tip_descendants_buffer = init_buffer(
            device,
            "Spring Tip Descendants",
            bytemuck::cast_slice(&tip_descendants),
        );
        self.rest_axes_buffer =
            init_buffer(device, "Spring Rest Axes", bytemuck::cast_slice(&rest_axes));
        self.bone_lengths_buffer = init_buffer(
            device,
            "Spring Bone Lengths",
            bytemuck::cast_slice(&bone_lengths),
        );
        let state_bytes = (total_slots.max(1) as usize) * 2 * std::mem::size_of::<[f32; 4]>();
        self.state_buffer = init_buffer(device, "Spring State", &vec![0u8; state_bytes]);
        self.chains_buffer = sized_buffer(
            device,
            "Spring Chains",
            (self.layouts.len().max(1) * std::mem::size_of::<GpuSpringChain>()) as u64,
        );
        self.reset_pending = true;
    }

    fn upload_chains(&mut self, queue: &wgpu::Queue, snapshot: &SkinnedAnimationSnapshot) {
        if self.layouts.is_empty() {
            return;
        }
        let reset = u32::from(self.reset_pending);
        let chains: Vec<GpuSpringChain> = self
            .layouts
            .iter()
            .map(|layout| {
                let runtime = snapshot
                    .spring_runtime
                    .get(&(layout.player, layout.chain_index))
                    .copied()
                    .unwrap_or([0.0; 8]);
                GpuSpringChain {
                    joint_start: layout.joint_start,
                    joint_count: layout.joint_count,
                    descendant_start: layout.descendant_start,
                    descendant_count: layout.descendant_count,
                    state_start: layout.state_start,
                    reset,
                    enabled: u32::from(runtime[5] > 0.0),
                    pad0: 0,
                    gravity: [runtime[0], runtime[1], runtime[2], 0.0],
                    stiffness: runtime[3].clamp(0.0, 1.0),
                    damping: runtime[4].clamp(0.0, 1.0),
                    dt: runtime[6],
                    pad1: 0.0,
                }
            })
            .collect();
        queue.write_buffer(&self.chains_buffer, 0, bytemuck::cast_slice(&chains));
        self.reset_pending = false;
    }

    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        bone_transforms_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinned Spring Bind Group"),
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
                    resource: self.joint_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.tip_descendants_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.rest_axes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.bone_lengths_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.state_buffer.as_entire_binding(),
                },
            ],
        })
    }
}
