use crate::pre_render::Renderer;
use crate::renderer::sprites::{SPRITES_PASS, SpriteFrameData, create_pass, hand_over};
use crate::renderer::{VulkanRenderer, install_scene_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_sprite::contract::render_data::SpriteFrame;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the sprites pass was not installed: {error}"),
    }
}

pub fn upload_sprites(mut renderer: Renderer, frame: Option<ResMut<SpriteFrame>>) {
    let Some(mut frame) = frame else {
        return;
    };
    let Some(view) = renderer.view.clone() else {
        return;
    };
    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == SPRITES_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() else {
        return;
    };
    let data = SpriteFrameData {
        instances: frame.instances.clone(),
        atlases: frame.atlases.clone(),
        uploads: std::mem::take(&mut frame.uploads),
        view: view.view,
        projection: view.projection,
        camera_right: [
            view.camera_right.x,
            view.camera_right.y,
            view.camera_right.z,
            0.0,
        ],
        camera_up: [view.camera_up.x, view.camera_up.y, view.camera_up.z, 0.0],
    };
    hand_over(pass, data);
}
