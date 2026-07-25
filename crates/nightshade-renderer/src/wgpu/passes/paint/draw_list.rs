//! Batched UI rectangle and text rendering pass.

mod constructor;
mod execute;
mod prepare;

use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use nalgebra_glm::Mat4;

const INITIAL_RECT_CAPACITY: usize = 4096;
const INITIAL_TEXT_VERTEX_CAPACITY: usize = 64000;
const INITIAL_TEXT_INDEX_CAPACITY: usize = 96000;
const INITIAL_TEXT_INSTANCE_CAPACITY: usize = 1024;
const INITIAL_CHARACTER_COLOR_CAPACITY: usize = 10000;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct PaintRectInstance {
    position_size: [f32; 4],
    color: [f32; 4],
    border_color: [f32; 4],
    clip_rect: [f32; 4],
    params: [f32; 4],
    shadow_color: [f32; 4],
    shadow_params: [f32; 4],
    effect_params: [f32; 4],
    quad_corner_01: [f32; 4],
    quad_corner_23: [f32; 4],
    effect_kind: u32,
    is_quad: u32,
    _padding: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct TextInstanceData {
    color: [f32; 4],
    outline_color: [f32; 4],
    clip_rect: [f32; 4],
    params: [f32; 4],
    position: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PaintPassVertex {
    position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UiTextVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    character_index: u32,
    text_instance_index: u32,
    _padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniforms {
    projection: Mat4,
}

struct TextDrawCall {
    index_start: u32,
    index_count: u32,
    layer: u8,
}

struct LayerDrawGroup {
    rect_start: u32,
    rect_count: u32,
    text_draw_start: usize,
    text_draw_count: usize,
}

/// A contiguous slab reservation within a GPU buffer.
#[derive(Clone, Copy, Debug)]
pub(super) struct SlabAlloc {
    /// First element index of the slab.
    pub start: u32,
    /// Number of elements reserved.
    pub capacity: u32,
}

/// Cached buffer allocations for one text slot, keyed by content signature.
#[derive(Clone, Debug)]
pub(super) struct TextSlotEntry {
    /// Hash of the slot's shaped content.
    pub signature: u64,
    /// Vertex slab reservation.
    pub vertex_alloc: SlabAlloc,
    /// Index slab reservation.
    pub index_alloc: SlabAlloc,
    /// Number of indices to draw.
    pub index_count: u32,
    /// Per-character color slab reservation.
    pub char_color_alloc: SlabAlloc,
}

/// Free-list allocator handing out slabs from a growing buffer.
#[derive(Default)]
pub(super) struct SlabAllocator {
    /// Reclaimed slabs available for reuse.
    pub free: Vec<SlabAlloc>,
    /// Next unused element index.
    pub high_water: u32,
}

impl SlabAllocator {
    /// Reserves a slab of `needed` elements, reusing a freed slab when one fits.
    pub fn alloc(&mut self, needed: u32) -> SlabAlloc {
        if needed == 0 {
            return SlabAlloc {
                start: 0,
                capacity: 0,
            };
        }
        if let Some(index) = self.free.iter().position(|s| s.capacity >= needed) {
            return self.free.swap_remove(index);
        }
        let start = self.high_water;
        self.high_water += needed;
        SlabAlloc {
            start,
            capacity: needed,
        }
    }

    /// Returns a slab to the free list for reuse.
    pub fn free(&mut self, alloc: SlabAlloc) {
        if alloc.capacity > 0 {
            self.free.push(alloc);
        }
    }
}

/// Render pass that draws batched UI rectangles and text.
pub struct PaintPass {
    rect_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,

    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,

    rect_quad_vertex_buffer: wgpu::Buffer,
    rect_quad_index_buffer: wgpu::Buffer,
    rect_instance_buffer: wgpu::Buffer,
    rect_draw_order_buffer: wgpu::Buffer,
    rect_instance_bind_group_layout: wgpu::BindGroupLayout,
    rect_instance_bind_group: wgpu::BindGroup,
    rect_instance_capacity: usize,
    rect_draw_order_capacity: usize,
    rect_count: u32,
    prev_rect_instances: Vec<PaintRectInstance>,

    text_vertex_buffer: wgpu::Buffer,
    text_index_buffer: wgpu::Buffer,
    text_instance_buffer: wgpu::Buffer,
    character_color_buffer: wgpu::Buffer,
    character_bg_color_buffer: wgpu::Buffer,
    text_data_bind_group_layout: wgpu::BindGroupLayout,
    text_data_bind_group: wgpu::BindGroup,
    text_font_bind_group_layout: wgpu::BindGroupLayout,
    text_font_bind_groups: Vec<wgpu::BindGroup>,
    _text_global_bind_group_layout: wgpu::BindGroupLayout,
    text_global_bind_group: wgpu::BindGroup,
    text_vertex_capacity: usize,
    text_index_capacity: usize,
    text_instance_capacity: usize,
    character_color_capacity: usize,
    character_bg_color_capacity: usize,
    prev_text_instance_data: Vec<TextInstanceData>,
    text_slots: std::collections::HashMap<u32, TextSlotEntry>,
    text_vertex_slab: SlabAllocator,
    text_index_slab: SlabAllocator,
    char_color_slab: SlabAllocator,
    prev_character_colors: Vec<[f32; 4]>,
    prev_character_bg_colors: Vec<[f32; 4]>,
    text_draw_calls: Vec<TextDrawCall>,
    layer_draw_groups: Vec<LayerDrawGroup>,

    cached_atlas_view: Option<wgpu::TextureView>,
    screen_width: f32,
    screen_height: f32,
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for PaintPass {
    fn name(&self) -> &'static str {
        "ui_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth"]
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        self.prepare_pass(device, queue, configs);
    }

    fn runs_in_compose_only_phase(&self) -> bool {
        true
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> Result<
        Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        crate::wgpu::rendergraph::RenderGraphError,
    > {
        self.execute_pass(context)
    }
}
