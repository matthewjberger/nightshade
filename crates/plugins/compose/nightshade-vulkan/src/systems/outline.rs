use crate::pre_render::Renderer;
use crate::renderer::outline::{OUTLINE_PASS, OutlineFrame, create_pass, hand_over};
use crate::renderer::{VulkanRenderer, install_swapchain_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_render::resources::outline::Outline;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_swapchain_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the outline pass was not installed: {error}"),
    }
}

pub fn upload_outline(mut renderer: Renderer, outline: Res<Outline>) {
    let named = outline
        .entities
        .iter()
        .chain(outline.active_entity.iter())
        .map(|entity| entity.id)
        .collect();
    let data = OutlineFrame {
        color: outline.color,
        named,
    };

    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == OUTLINE_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(pass, data);
    }
}
