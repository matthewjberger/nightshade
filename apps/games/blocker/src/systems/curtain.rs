use crate::curtain::{CURTAIN_PASS, CurtainPass};
use blocker_screen::data::screen::Screen;
use blocker_world::resources::{Arrival, Generation};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::State;

pub fn install_curtain_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: nightshade_wgpu::backend::PassBuild,
    _color_format: nightshade_wgpu::wgpu::TextureFormat,
) {
    let color = build.targets.scene_color;
    let depth = build.targets.depth;
    let pass = CurtainPass::new(build);
    if let Err(error) = nightshade_wgpu::backend::install_lit_pass(
        graph,
        Box::new(pass),
        &[("color", color), ("depth", depth)],
    ) {
        tracing::error!("the loading curtain was not installed: {error}");
    }
}

pub fn draw_the_curtain(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    time: Res<Time>,
    state: Res<State<Screen>>,
    arrival: Res<Arrival>,
    generation: Res<Generation>,
) {
    let Some(pass) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, CURTAIN_PASS)
    else {
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<CurtainPass>() else {
        return;
    };
    let shown = state.current == Screen::Loading;
    if shown && !pass.shown {
        pass.time = 0.0;
    }
    pass.shown = shown;
    if !shown {
        return;
    }
    pass.time += time.delta_time;
    pass.progress = if arrival.wanted == 0 {
        0.0
    } else {
        arrival.landed as f32 / arrival.wanted as f32
    };
    pass.seed = (generation.seed % 1024) as f32 / 1024.0;
}
