pub mod pass;

use crate::data;
use crate::resources::MashupResources;
use nightshade::prelude::*;
use nightshade_wgpu::RenderAppExt;
use pass::{PortalDraw, PortalPass};

pub struct MashupWgpuPlugin;

impl Plugin for MashupWgpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(install);
        app.on_pre_render(hand_over);
    }
}

fn install(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: nightshade_wgpu::backend::PassBuild,
    _color_format: nightshade_wgpu::wgpu::TextureFormat,
) {
    let color = build.targets.scene_color;
    let depth = build.targets.depth;
    if let Err(error) = nightshade_wgpu::backend::install_lit_pass(
        graph,
        Box::new(pass::new(build)),
        &[("color", color), ("depth", depth)],
    ) {
        tracing::error!("the portal pass was not installed: {error}");
    }
}

fn hand_over(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    time: Res<nightshade_platform::contract::time::Time>,
    resources: Res<MashupResources>,
) {
    let player_camera = resources.camera;
    let mut draws = Vec::new();
    for frame in resources.frames.iter().flatten() {
        let Some(viewport) = renderer.camera_viewports.get(&frame.eye) else {
            continue;
        };
        draws.push(PortalDraw {
            model: frame.model,
            view: viewport.view.clone(),
            rim: frame.rim,
            fade: frame.fade,
        });
    }

    let Some(node) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, data::PORTAL_PASS)
    else {
        return;
    };
    let Some(node) = (node as &mut dyn std::any::Any).downcast_mut::<PortalPass>() else {
        return;
    };
    pass::hand_over(node, player_camera, time.uptime_seconds as f32, draws);
}
