use crate::pre_render::Renderer;
use crate::renderer::grid::{GRID_PASS, GridFrame, create_pass, hand_over};
use crate::renderer::{VulkanRenderer, install_scene_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_render::data::config::DebugDraw;
use nightshade_render::queries::projection::matrix_columns;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the grid pass was not installed: {error}"),
    }
}

pub fn upload_grid(mut renderer: Renderer, debug: Res<DebugDraw>) {
    let visible = debug.show_grid;
    let Some(view) = renderer.view.clone() else {
        return;
    };
    let data = GridFrame {
        view_projection: matrix_columns(&view.view_projection),
        camera_position: [
            view.camera_position.x,
            view.camera_position.y,
            view.camera_position.z,
        ],
        parallel: false,
        parallel_scale: 1.0,
        visible,
    };

    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == GRID_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(pass, data);
    }
}
