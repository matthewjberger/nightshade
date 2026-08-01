use ash::vk;

use crate::renderer::context::Context;
use crate::renderer::pass::{Pass, PassRecord, PassSetup, create_shader_module};

pub const GRID_PASS: &str = "grid";

const REACH: f32 = 500.0;
const SMALLEST_CELL_PIXELS: f32 = 2.0;
const FINEST_CELL: f32 = 1.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct GridParams {
    view_projection: [[f32; 4]; 4],
    camera: [f32; 4],
    settings: [f32; 4],
}

#[derive(Default)]
pub struct GridFrame {
    pub view_projection: [[f32; 4]; 4],
    pub camera_position: [f32; 3],
    pub parallel: bool,
    pub parallel_scale: f32,
    pub visible: bool,
}

pub struct GridPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    data: GridFrame,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<GridPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<GridParams>() as u32);
    let layout_info =
        vk::PipelineLayoutCreateInfo::default().push_constant_ranges(std::slice::from_ref(&range));
    let pipeline_layout = unsafe { context.device.create_pipeline_layout(&layout_info, None)? };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    Ok(GridPass {
        pipeline,
        pipeline_layout,
        data: GridFrame::default(),
    })
}

pub fn hand_over(pass: &mut GridPass, data: GridFrame) {
    pass.data = data;
}

impl Pass for GridPass {
    fn name(&self) -> &'static str {
        GRID_PASS
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if !self.data.visible {
            return;
        }
        let params = GridParams {
            view_projection: self.data.view_projection,
            camera: [
                self.data.camera_position[0],
                self.data.camera_position[1],
                self.data.camera_position[2],
                REACH,
            ],
            settings: [
                SMALLEST_CELL_PIXELS,
                FINEST_CELL,
                if self.data.parallel { 1.0 } else { 0.0 },
                self.data.parallel_scale,
            ],
        };
        let device = &record.context.device;
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            device.cmd_push_constants(
                record.command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&params),
            );
            device.cmd_draw(record.command_buffer, 6, 1, 0, 0);
        }
    }

    fn destroy(&mut self, context: &Context) {
        unsafe {
            context.device.destroy_pipeline(self.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/grid.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/grid.frag.spv"))?;
    let entry = c"main";
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(entry),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(entry),
    ];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(false)
        .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
    let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD);
    let normal_blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::empty());
    let attachments = [
        blend_attachment,
        normal_blend,
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
    ];
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments);

    let color_formats = [setup.hdr_format, setup.normal_format, setup.motion_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&assembly)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    unsafe {
        context.device.destroy_shader_module(vertex, None);
        context.device.destroy_shader_module(fragment, None);
    }
    Ok(created[0])
}
