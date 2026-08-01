use crate::renderer::context::Context;
use crate::renderer::glyph_atlas::GlyphAtlas;
use crate::renderer::resource::{self, Resource, ResourceStates, Usage};
use ash::vk;
use nightshade_render::data::config::RenderView;

pub struct PassSetup<'a> {
    pub context: &'a Context,
    pub color_format: vk::Format,
    pub hdr_format: vk::Format,
    pub normal_format: vk::Format,
    pub motion_format: vk::Format,
    pub entity_id_format: vk::Format,
    pub accumulated_format: vk::Format,
    pub revealed_format: vk::Format,
    pub depth_format: vk::Format,
    pub frames_in_flight: usize,
    pub extent: vk::Extent2D,
    pub screen: vk::Extent2D,
    pub color: vk::ImageView,
    pub depth: vk::ImageView,
    pub normal: vk::ImageView,
    pub motion: vk::ImageView,
    pub entity_id: vk::ImageView,
    pub accumulated: vk::ImageView,
    pub revealed: vk::ImageView,
    pub glyphs: &'a GlyphAtlas,
}

#[derive(Clone, Copy)]
pub struct ScreenTargets {
    pub extent: vk::Extent2D,
    pub screen: vk::Extent2D,
    pub color: vk::ImageView,
    pub depth: vk::ImageView,
    pub normal: vk::ImageView,
    pub motion: vk::ImageView,
    pub entity_id: vk::ImageView,
    pub accumulated: vk::ImageView,
    pub revealed: vk::ImageView,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeKind {
    EarlyDepth,
    Pyramid,
    LateDepth,
    Gather,
    Color,
    Grab,
    Refracted,
    Blended,
    OverSolids,
    Decal,
    Post,
    Swapchain,
}

pub struct PassRecord<'a> {
    pub context: &'a Context,
    pub command_buffer: vk::CommandBuffer,
    pub extent: vk::Extent2D,
    pub frame_index: usize,
    pub view: Option<&'a RenderView>,
    pub scope: ScopeKind,
    pub states: &'a mut ResourceStates,
    pub glyphs: &'a GlyphAtlas,
    pub air: &'a crate::renderer::scene::fog::Air,
    pub probes: &'a crate::renderer::scene::probes::Probes,
}

impl PassRecord<'_> {
    pub fn require(&mut self, resource: Resource, usage: Usage) {
        resource::require(
            self.states,
            &self.context.device,
            self.command_buffer,
            resource,
            usage,
        );
    }

    pub fn discard(&mut self, resource: Resource) {
        resource::discard(self.states, resource);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub enum PassStage {
    Sky,
    #[default]
    Scene,
    Decal,
    Post,
    Swapchain,
}

pub trait Pass: std::any::Any + Send {
    fn name(&self) -> &'static str;
    fn usage(&self, _scope: ScopeKind) -> &[(Resource, Usage)] {
        &[]
    }
    fn register(&self, _states: &mut ResourceStates) {}
    fn compute(&mut self, _record: &mut PassRecord) {}
    fn offscreen(&mut self, _record: &mut PassRecord) {}
    fn draw(&mut self, _record: &mut PassRecord) {}
    fn has_depth_prepass(&self) -> bool {
        false
    }
    fn between(&mut self, _record: &mut PassRecord) {}
    fn resize(&mut self, _context: &Context, _targets: ScreenTargets) {}
    fn retarget(&mut self, _setup: &PassSetup<'_>) {}
    fn destroy(&mut self, _context: &Context) {}
}

pub fn read_spirv(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let mut cursor = std::io::Cursor::new(bytes);
    Ok(ash::util::read_spv(&mut cursor)?)
}

pub const REACHED_ON_ANY_QUEUE: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::ALL_COMMANDS;

pub fn create_shader_module(
    context: &Context,
    bytes: &[u8],
) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
    let code = read_spirv(bytes)?;
    let create_info = vk::ShaderModuleCreateInfo::default().code(&code);
    Ok(unsafe { context.device.create_shader_module(&create_info, None)? })
}
