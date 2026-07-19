use crate::config::{SkinnedAnimationSnapshot, SkinnedChannelData};
use crate::skinning::SkinningCache;
use nightshade_ecs::Entity;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuAnimBone {
    output_index: u32,
    parent_output_index: i32,
    cur_channel_start: u32,
    cur_channel_count: u32,
    blend_channel_start: u32,
    blend_channel_count: u32,
    layer_start: u32,
    layer_count: u32,
    rest_translation: [f32; 4],
    rest_rotation: [f32; 4],
    rest_scale: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuAnimBoneLayer {
    channel_start: u32,
    channel_count: u32,
    runtime_index: u32,
    pad0: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuAnimChannel {
    property: u32,
    interpolation: u32,
    input_offset: u32,
    key_count: u32,
    output_offset: u32,
    output_stride: u32,
    pad0: u32,
    pad1: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuAnimSkeleton {
    bone_start: u32,
    bone_count: u32,
    time: f32,
    blend_from_time: f32,
    blend_factor: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    armature_root: [[f32; 4]; 4],
}

struct SkeletonLayout {
    skin_entity: Entity,
    player_entity: Entity,
    bone_start: u32,
    bone_count: u32,
}

#[derive(Default)]
struct FlatChannels {
    channels: Vec<GpuAnimChannel>,
    times: Vec<f32>,
    values: Vec<[f32; 4]>,
}

impl FlatChannels {
    fn push_channel(&mut self, channel: &SkinnedChannelData) {
        let input_offset = self.times.len() as u32;
        self.times.extend_from_slice(&channel.input);
        let output_offset = self.values.len() as u32;
        self.values.extend_from_slice(&channel.values);
        self.channels.push(GpuAnimChannel {
            property: channel.property,
            interpolation: channel.interpolation,
            input_offset,
            key_count: channel.input.len() as u32,
            output_offset,
            output_stride: channel.stride,
            pad0: 0,
            pad1: 0,
        });
    }
}

pub(super) struct SkinnedAnimationGpu {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    skeletons_buffer: wgpu::Buffer,
    bones_buffer: wgpu::Buffer,
    channels_buffer: wgpu::Buffer,
    times_buffer: wgpu::Buffer,
    values_buffer: wgpu::Buffer,
    bone_layers_buffer: wgpu::Buffer,
    layer_runtime_buffer: wgpu::Buffer,
    runtime_slots: Vec<(Entity, usize)>,
    layouts: Vec<SkeletonLayout>,
    cached_static_signature: u64,
    bone_transforms_generation: u64,
}

fn storage_buffer(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
    let fallback = [0u8; 16];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: if data.is_empty() { &fallback } else { data },
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

impl SkinnedAnimationGpu {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "animation_compute.wgsl",
            include_str!("../../../shaders/animation_compute.wgsl"),
        );

        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..8)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding != 5,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Animation Compute Bind Group Layout"),
            entries: &entries,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Animation Compute Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Animation Compute Pipeline"),
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
            skeletons_buffer: storage_buffer(device, "Anim Skeletons", &[]),
            bones_buffer: storage_buffer(device, "Anim Bones", &[]),
            channels_buffer: storage_buffer(device, "Anim Channels", &[]),
            times_buffer: storage_buffer(device, "Anim Times", &[]),
            values_buffer: storage_buffer(device, "Anim Values", &[]),
            bone_layers_buffer: storage_buffer(device, "Anim Bone Layers", &[]),
            layer_runtime_buffer: storage_buffer(device, "Anim Layer Runtime", &[]),
            runtime_slots: Vec::new(),
            layouts: Vec::new(),
            cached_static_signature: 0,
            bone_transforms_generation: u64::MAX,
        }
    }

    pub(super) fn skeleton_count(&self) -> u32 {
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

        self.upload_skeletons(queue, snapshot);
        self.upload_layer_runtime(queue, snapshot);
    }

    fn rebuild_static(
        &mut self,
        device: &wgpu::Device,
        snapshot: &SkinnedAnimationSnapshot,
        skinning_cache: &SkinningCache,
    ) {
        let mut bones: Vec<GpuAnimBone> = Vec::new();
        let mut bone_layers: Vec<GpuAnimBoneLayer> = Vec::new();
        let mut flat = FlatChannels::default();
        let mut layouts: Vec<SkeletonLayout> = Vec::new();
        self.runtime_slots.clear();

        for skeleton in &snapshot.skeletons {
            let Some(&skin_index) = skinning_cache
                .entity_skin_indices
                .get(&skeleton.skin_entity)
            else {
                continue;
            };
            let base_bone_index = skinning_cache.get_base_bone_index(skin_index);

            let layer_slot_base = self.runtime_slots.len() as u32;
            for layer_index in 0..skeleton.layer_count as usize {
                self.runtime_slots
                    .push((skeleton.player_entity, layer_index));
            }

            let bone_start = bones.len() as u32;
            for joint in &skeleton.joints_ordered {
                let parent_output_index = match joint.parent_local {
                    Some(parent) => (base_bone_index + parent) as i32,
                    None => -1,
                };

                let (cur_start, cur_count) = append_joint_channels(&mut flat, &joint.cur_channels);
                let (blend_start, blend_count) =
                    append_joint_channels(&mut flat, &joint.blend_channels);

                let layer_start = bone_layers.len() as u32;
                let mut layer_count = 0u32;
                for (layer_index, channels) in joint.layer_channels.iter().enumerate() {
                    let (start, count) = append_joint_channels(&mut flat, channels);
                    if count == 0 {
                        continue;
                    }
                    bone_layers.push(GpuAnimBoneLayer {
                        channel_start: start,
                        channel_count: count,
                        runtime_index: layer_slot_base + layer_index as u32,
                        pad0: 0,
                    });
                    layer_count += 1;
                }

                bones.push(GpuAnimBone {
                    output_index: base_bone_index + joint.local_index,
                    parent_output_index,
                    cur_channel_start: cur_start,
                    cur_channel_count: cur_count,
                    blend_channel_start: blend_start,
                    blend_channel_count: blend_count,
                    layer_start,
                    layer_count,
                    rest_translation: [
                        joint.rest_translation[0],
                        joint.rest_translation[1],
                        joint.rest_translation[2],
                        0.0,
                    ],
                    rest_rotation: joint.rest_rotation,
                    rest_scale: [
                        joint.rest_scale[0],
                        joint.rest_scale[1],
                        joint.rest_scale[2],
                        0.0,
                    ],
                });
            }

            layouts.push(SkeletonLayout {
                skin_entity: skeleton.skin_entity,
                player_entity: skeleton.player_entity,
                bone_start,
                bone_count: skeleton.joint_count,
            });
        }

        self.layouts = layouts;
        self.bones_buffer = storage_buffer(device, "Anim Bones", bytemuck::cast_slice(&bones));
        self.channels_buffer = storage_buffer(
            device,
            "Anim Channels",
            bytemuck::cast_slice(&flat.channels),
        );
        self.times_buffer = storage_buffer(device, "Anim Times", bytemuck::cast_slice(&flat.times));
        self.values_buffer =
            storage_buffer(device, "Anim Values", bytemuck::cast_slice(&flat.values));
        self.bone_layers_buffer = storage_buffer(
            device,
            "Anim Bone Layers",
            bytemuck::cast_slice(&bone_layers),
        );
        let runtime_bytes = self.runtime_slots.len().max(1) * std::mem::size_of::<[f32; 2]>();
        self.layer_runtime_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Anim Layer Runtime"),
            size: runtime_bytes as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let skeleton_bytes = self.layouts.len().max(1) * std::mem::size_of::<GpuAnimSkeleton>();
        self.skeletons_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Anim Skeletons"),
            size: skeleton_bytes as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    fn upload_skeletons(&mut self, queue: &wgpu::Queue, snapshot: &SkinnedAnimationSnapshot) {
        if self.layouts.is_empty() {
            return;
        }
        let mut skeletons: Vec<GpuAnimSkeleton> = Vec::with_capacity(self.layouts.len());
        for layout in &self.layouts {
            let (bone_count, time, blend_from_time, blend_factor) =
                match snapshot.player_runtime.get(&layout.player_entity) {
                    Some(runtime) => (layout.bone_count, runtime[0], runtime[1], runtime[2]),
                    None => (0, 0.0, 0.0, 1.0),
                };
            let armature_root = snapshot
                .armature_roots
                .get(&layout.skin_entity)
                .copied()
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            skeletons.push(GpuAnimSkeleton {
                bone_start: layout.bone_start,
                bone_count,
                time,
                blend_from_time,
                blend_factor,
                pad0: 0,
                pad1: 0,
                pad2: 0,
                armature_root: matrix_to_array(armature_root),
            });
        }

        queue.write_buffer(&self.skeletons_buffer, 0, bytemuck::cast_slice(&skeletons));
    }

    fn upload_layer_runtime(&self, queue: &wgpu::Queue, snapshot: &SkinnedAnimationSnapshot) {
        if self.runtime_slots.is_empty() {
            return;
        }
        let runtime: Vec<[f32; 2]> = self
            .runtime_slots
            .iter()
            .map(|(player_entity, layer_index)| {
                snapshot
                    .layer_runtime
                    .get(&(*player_entity, *layer_index))
                    .copied()
                    .unwrap_or([0.0, 0.0])
            })
            .collect();
        queue.write_buffer(
            &self.layer_runtime_buffer,
            0,
            bytemuck::cast_slice(&runtime),
        );
    }

    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        bone_transforms_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animation Compute Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.skeletons_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.bones_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.channels_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.times_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.values_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: bone_transforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.bone_layers_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.layer_runtime_buffer.as_entire_binding(),
                },
            ],
        })
    }
}

fn append_joint_channels(flat: &mut FlatChannels, channels: &[SkinnedChannelData]) -> (u32, u32) {
    let start = flat.channels.len() as u32;
    let mut count = 0u32;
    for channel in channels {
        flat.push_channel(channel);
        count += 1;
    }
    (start, count)
}

fn matrix_to_array(matrix: nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let mut output = [[0.0f32; 4]; 4];
    for column in 0..4 {
        for row in 0..4 {
            output[column][row] = matrix[(row, column)];
        }
    }
    output
}
