use nightshade_wgpu::wgpu;
mod constructor;
mod execute;
mod prepare;

use nalgebra_glm::Mat4;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition, PassStage};

const INITIAL_RECT_CAPACITY: usize = 4096;
const INITIAL_TEXT_VERTEX_CAPACITY: usize = 64000;
const INITIAL_TEXT_INDEX_CAPACITY: usize = 96000;
const INITIAL_TEXT_INSTANCE_CAPACITY: usize = 1024;
const INITIAL_CHARACTER_COLOR_CAPACITY: usize = 10000;

use nightshade_paint::{PaintRectInstance, TextInstanceData};

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
    flags: u32,
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

#[derive(Clone, Copy, Debug)]
pub(super) struct SlabAlloc {
    pub start: u32,
    pub capacity: u32,
}

#[derive(Clone, Debug)]
pub(super) struct TextSlotEntry {
    pub signature: u64,
    pub vertex_alloc: SlabAlloc,
    pub index_alloc: SlabAlloc,
    pub index_count: u32,
    pub char_color_alloc: SlabAlloc,
}

#[derive(Default)]
pub(super) struct SlabAllocator {
    pub free: Vec<SlabAlloc>,
    pub high_water: u32,
}

impl SlabAllocator {
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

    pub fn free(&mut self, alloc: SlabAlloc) {
        if alloc.capacity > 0 {
            self.free.push(alloc);
        }
    }
}

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
    draw_list: nightshade_paint::PaintDrawList,
}

impl PaintPass {
    pub fn swap_draw_list(
        &mut self,
        draw_list: nightshade_paint::PaintDrawList,
    ) -> nightshade_paint::PaintDrawList {
        std::mem::replace(&mut self.draw_list, draw_list)
    }
}

const PAINT_PASS_DECL: PassDecl = PassDecl {
    name: super::PAINT_PASS,
    reads_writes: &["color", "depth"],
    optional_reads: &["glyph_atlas"],
    stage: PassStage::Swapchain,
    position: PassPosition::Swapchain,
    compose_only_phase: true,
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for PaintPass {
    fn decl(&self) -> &PassDecl {
        &PAINT_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.release_glyph_atlas();
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        self.prepare_pass(device, queue, configs);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> Result<(), nightshade_wgpu::backend::RenderGraphError> {
        if self.needs_glyph_atlas()
            && let Ok(atlas_view) = context.get_texture_view("glyph_atlas")
        {
            self.update_glyph_atlas(context.device, atlas_view);
        }
        self.execute_pass(context)
    }
}
