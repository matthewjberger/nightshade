use crate::commands::stand_the_graph;
use crate::resources::Board;
use crate::theme;
use nalgebra_glm::vec2;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::commands::scene::spawn_ortho_camera;
use nightshade_camera::resources::ActiveCamera;
use nightshade_render::prelude::RenderSettings;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::{RetainedUiRuntime, UiTreeBuilder};
use nightshade_ui_nodegraph::prelude::{GraphView, spread_the_canvas};

pub fn stand_it_up(
    mut settings: ResMut<RenderSettings>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut active_camera: ResMut<ActiveCamera>,
    mut board: ResMut<Board>,
    mut commands: Commands,
) {
    settings.render_world_to_swapchain = false;
    settings.clear_color = theme::CLEAR;
    runtime.background_color = None;
    active_camera.0 = Some(spawn_ortho_camera(&mut commands, vec2(0.0, 0.0)));
    board.standing = stand_the_graph(&mut commands);
}

pub fn spread_the_board(mut view: ResMut<GraphView>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    spread_the_canvas(&mut tree, &mut view);
    tree.finish();
}
