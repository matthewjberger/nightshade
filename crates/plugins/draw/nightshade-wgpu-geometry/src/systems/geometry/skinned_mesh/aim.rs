use nightshade::ecs::Entity;
use nightshade_render::data::config::SkinnedAnimationSnapshot;
use nightshade_render::data::skinning::SkinningCache;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuAimConstraint {
    joint_start: u32,
    joint_count: u32,
    descendant_start: u32,
    descendant_count: u32,
    goal: [f32; 4],
    forward: [f32; 4],
    weight: f32,
    cone_angle: f32,
    pad0: u32,
    pad1: u32,
}

struct AimConstraintLayout {
    player: Entity,
    chain_index: usize,
    joint_start: u32,
    joint_count: u32,
    descendant_start: u32,
    descendant_count: u32,
    forward: [f32; 3],
}

pub(super) struct SkinnedAimGpu {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    constraints_buffer: wgpu::Buffer,
    joint_indices_buffer: wgpu::Buffer,
    tip_descendants_buffer: wgpu::Buffer,
    layouts: Vec<AimConstraintLayout>,
    cached_static_signature: u64,
    cached_clip_signature: u64,
    bone_transforms_generation: u64,
}

fn sized_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    build::storage(device, label, size.max(16))
}

fn init_buffer(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
    let fallback = [0u8; 16];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: if data.is_empty() { &fallback } else { data },
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

impl SkinnedAimGpu {
    pub(super) fn new(build: PassBuild) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader = composer.compile_wgsl(
            device,
            "aim_compute.wgsl",
            include_str!("../../../shaders/aim_compute.wgsl"),
        );

        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..4)
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
        let bind_group_layout =
            build::bind_layout(device, "Skinned Aim Bind Group Layout", &entries);

        let pipeline_layout =
            build::pipeline_layout(device, "Skinned Aim Pipeline Layout", &[&bind_group_layout]);

        let pipeline = build::compute(
            device,
            "Skinned Aim Pipeline",
            &shader,
            "main",
            &pipeline_layout,
        );

        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            constraints_buffer: sized_buffer(device, "Aim Constraints", 16),
            joint_indices_buffer: sized_buffer(device, "Aim Joint Indices", 16),
            tip_descendants_buffer: sized_buffer(device, "Aim Tip Descendants", 16),
            layouts: Vec::new(),
            cached_static_signature: u64::MAX,
            cached_clip_signature: u64::MAX,
            bone_transforms_generation: u64::MAX,
        }
    }

    pub(super) fn constraint_count(&self) -> u32 {
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
        configs: &nightshade_wgpu::backend::RenderInputs,
        skinning_cache: &SkinningCache,
        bone_transforms_buffer: &wgpu::Buffer,
        bone_transforms_generation: u64,
    ) {
        let snapshot = &configs.scene.render_animation;

        if snapshot.signature != self.cached_static_signature
            || snapshot.clip_signature != self.cached_clip_signature
            || self.bone_transforms_generation != bone_transforms_generation
        {
            self.rebuild_static(device, snapshot, skinning_cache);
            self.cached_static_signature = snapshot.signature;
            self.cached_clip_signature = snapshot.clip_signature;
            self.bone_transforms_generation = bone_transforms_generation;
            self.bind_group = Some(self.create_bind_group(device, bone_transforms_buffer));
        }

        self.upload_constraints(queue, snapshot);
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

        for skeleton in &snapshot.skeletons {
            if skeleton.aim_constraints.is_empty() {
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

            for constraint in &skeleton.aim_constraints {
                if constraint.joint_locals.is_empty() {
                    continue;
                }
                let joint_start = joint_indices.len() as u32;
                for &local in &constraint.joint_locals {
                    joint_indices.push(base_bone_index + local);
                }
                let joint_count = constraint.joint_locals.len() as u32;

                let tip_local = *constraint.joint_locals.last().unwrap();
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

                self.layouts.push(AimConstraintLayout {
                    player: skeleton.player_entity,
                    chain_index: constraint.chain_index as usize,
                    joint_start,
                    joint_count,
                    descendant_start,
                    descendant_count,
                    forward: constraint.forward,
                });
            }
        }

        self.joint_indices_buffer = init_buffer(
            device,
            "Aim Joint Indices",
            bytemuck::cast_slice(&joint_indices),
        );
        self.tip_descendants_buffer = init_buffer(
            device,
            "Aim Tip Descendants",
            bytemuck::cast_slice(&tip_descendants),
        );
        self.constraints_buffer = sized_buffer(
            device,
            "Aim Constraints",
            (self.layouts.len().max(1) * std::mem::size_of::<GpuAimConstraint>()) as u64,
        );
    }

    fn upload_constraints(&self, queue: &wgpu::Queue, snapshot: &SkinnedAnimationSnapshot) {
        if self.layouts.is_empty() {
            return;
        }
        let constraints: Vec<GpuAimConstraint> = self
            .layouts
            .iter()
            .map(|layout| {
                let runtime = snapshot
                    .aim_runtime
                    .get(&(layout.player, layout.chain_index))
                    .copied()
                    .unwrap_or([0.0; 8]);
                let weight = if runtime[4] > 0.0 { runtime[3] } else { 0.0 };
                let cone_angle = if runtime[5] > 0.0 {
                    runtime[5]
                } else {
                    std::f32::consts::PI
                };
                GpuAimConstraint {
                    joint_start: layout.joint_start,
                    joint_count: layout.joint_count,
                    descendant_start: layout.descendant_start,
                    descendant_count: layout.descendant_count,
                    goal: [runtime[0], runtime[1], runtime[2], 0.0],
                    forward: [layout.forward[0], layout.forward[1], layout.forward[2], 0.0],
                    weight,
                    cone_angle,
                    pad0: 0,
                    pad1: 0,
                }
            })
            .collect();
        queue.write_buffer(
            &self.constraints_buffer,
            0,
            bytemuck::cast_slice(&constraints),
        );
    }

    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        bone_transforms_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        build::binding(
            device,
            "Skinned Aim Bind Group",
            &self.bind_group_layout,
            vec![
                bone_transforms_buffer.as_entire_binding(),
                self.constraints_buffer.as_entire_binding(),
                self.joint_indices_buffer.as_entire_binding(),
                self.tip_descendants_buffer.as_entire_binding(),
            ],
        )
    }
}
