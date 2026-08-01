use ash::vk;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Resource {
    Swapchain,
    Depth,
    Hdr,
    Normal,
    Motion,
    EntityId,
    Accumulated,
    Revealed,
    ShadowAtlas,
    LocalShadowAtlas,
    Pyramid,
    Grab,
    Bloom(usize),
    Occlusion(usize),
    Reflection,
    ReflectionKept(usize),
    ReflectionMoments(usize),
    ReflectionWave(usize),
    ReflectionNoise(usize),
    ReflectionSoft,
    Indirect,
    IndirectSoft,
    Resolved,
    Kept(usize),
    Focus,
    Composed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Usage {
    ColorAttachment,
    DepthAttachment,
    Sampled,
    SampledByCompute,
    SampledByCull,
    SampledByComputeAndFragment,
    StorageWrite,
    TransferRead,
    Present,
}

impl Usage {
    fn layout(self) -> vk::ImageLayout {
        match self {
            Usage::ColorAttachment => vk::ImageLayout::ATTACHMENT_OPTIMAL,
            Usage::DepthAttachment => vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            Usage::Sampled
            | Usage::SampledByCompute
            | Usage::SampledByCull
            | Usage::SampledByComputeAndFragment => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            Usage::StorageWrite => vk::ImageLayout::GENERAL,
            Usage::TransferRead => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            Usage::Present => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }

    fn access(self) -> vk::AccessFlags2 {
        match self {
            Usage::ColorAttachment => {
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE | vk::AccessFlags2::COLOR_ATTACHMENT_READ
            }
            Usage::DepthAttachment => {
                vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
            }
            Usage::Sampled
            | Usage::SampledByCompute
            | Usage::SampledByCull
            | Usage::SampledByComputeAndFragment => vk::AccessFlags2::SHADER_SAMPLED_READ,
            Usage::StorageWrite => vk::AccessFlags2::SHADER_STORAGE_WRITE,
            Usage::TransferRead => vk::AccessFlags2::TRANSFER_READ,
            Usage::Present => vk::AccessFlags2::empty(),
        }
    }

    fn stage(self) -> vk::PipelineStageFlags2 {
        match self {
            Usage::ColorAttachment => vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            Usage::DepthAttachment => {
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
            }
            Usage::Sampled => vk::PipelineStageFlags2::FRAGMENT_SHADER,
            Usage::SampledByCompute | Usage::StorageWrite => {
                vk::PipelineStageFlags2::COMPUTE_SHADER
            }
            Usage::SampledByCull => {
                vk::PipelineStageFlags2::COMPUTE_SHADER | vk::PipelineStageFlags2::TASK_SHADER_EXT
            }
            Usage::SampledByComputeAndFragment => {
                vk::PipelineStageFlags2::COMPUTE_SHADER | vk::PipelineStageFlags2::FRAGMENT_SHADER
            }
            Usage::TransferRead => vk::PipelineStageFlags2::COPY,
            Usage::Present => vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
        }
    }

    pub fn keeps(self) -> bool {
        matches!(
            self,
            Usage::Sampled
                | Usage::SampledByCompute
                | Usage::SampledByCull
                | Usage::SampledByComputeAndFragment
                | Usage::TransferRead
                | Usage::Present
        )
    }
}

#[derive(Clone, Copy)]
struct State {
    layout: vk::ImageLayout,
    access: vk::AccessFlags2,
    stage: vk::PipelineStageFlags2,
}

struct Entry {
    image: vk::Image,
    aspect: vk::ImageAspectFlags,
    state: State,
}

#[derive(Default)]
pub struct ResourceStates {
    entries: HashMap<Resource, Entry>,
}

pub fn register(
    states: &mut ResourceStates,
    resource: Resource,
    image: vk::Image,
    aspect: vk::ImageAspectFlags,
) {
    states.entries.insert(
        resource,
        Entry {
            image,
            aspect,
            state: State {
                layout: vk::ImageLayout::UNDEFINED,
                access: vk::AccessFlags2::empty(),
                stage: vk::PipelineStageFlags2::TOP_OF_PIPE,
            },
        },
    );
}

pub fn discard(states: &mut ResourceStates, resource: Resource) {
    if let Some(entry) = states.entries.get_mut(&resource) {
        entry.state.layout = vk::ImageLayout::UNDEFINED;
    }
}

pub fn require(
    states: &mut ResourceStates,
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    resource: Resource,
    usage: Usage,
) {
    let Some(entry) = states.entries.get_mut(&resource) else {
        return;
    };
    let wanted = State {
        layout: usage.layout(),
        access: usage.access(),
        stage: usage.stage(),
    };
    if entry.state.layout == wanted.layout && !writes(entry.state.access) {
        return;
    }

    let barrier = vk::ImageMemoryBarrier2::default()
        .image(entry.image)
        .old_layout(entry.state.layout)
        .new_layout(wanted.layout)
        .src_access_mask(entry.state.access)
        .dst_access_mask(wanted.access)
        .src_stage_mask(entry.state.stage)
        .dst_stage_mask(wanted.stage)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: entry.aspect,
            base_mip_level: 0,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS,
        });
    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };

    entry.state = wanted;
}

fn writes(access: vk::AccessFlags2) -> bool {
    access.intersects(
        vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
            | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
            | vk::AccessFlags2::SHADER_STORAGE_WRITE,
    )
}
