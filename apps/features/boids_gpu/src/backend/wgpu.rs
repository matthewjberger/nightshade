pub mod pass;

use crate::data::{BOIDS_GPU_PASS, MAX_BOID_COUNT};
use crate::queries::bounds_for;
use crate::resources::{BoidsGpuResources, FlockSettings};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_wgpu::RenderAppExt;
use pass::BoidsGpuPass;

pub struct BoidsWgpuPlugin;

impl Plugin for BoidsWgpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(install_boids_pass);
        app.on_pre_render(hand_over_flock);
    }
}

fn install_boids_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: nightshade_wgpu::backend::PassBuild,
    _color_format: nightshade_wgpu::wgpu::TextureFormat,
) {
    let color = build.targets.scene_color;
    let depth = build.targets.depth;
    let pass = BoidsGpuPass::new(build, MAX_BOID_COUNT as u32);
    if let Err(error) = nightshade_wgpu::backend::install_scene_pass(
        graph,
        Box::new(pass),
        &[("color", color), ("depth", depth)],
    ) {
        tracing::error!("boids pass not installed: {error}");
    }
}

fn hand_over_flock(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    time: Res<Time>,
    mut flock: ResMut<BoidsGpuResources>,
) {
    let delta_time = time.delta_time;
    let (count, visible, seeded) = (flock.count, flock.visible, flock.seeded);
    let bounds = bounds_for(count);

    let queue = renderer.queue.clone();
    let Some(pass) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, BOIDS_GPU_PASS)
    else {
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<BoidsGpuPass>() else {
        return;
    };

    if !seeded {
        pass.seed(&queue, count as u32, bounds);
        flock.seeded = true;
    }

    pass.hand_over(FlockSettings {
        delta_time,
        bounds,
        count: count as u32,
        visible,
        ..Default::default()
    });
}
