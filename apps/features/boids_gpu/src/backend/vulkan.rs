pub mod pass;

use crate::data::{BOIDS_GPU_PASS, MAX_BOID_COUNT};
use crate::queries::bounds_for;
use crate::resources::{BoidsGpuResources, FlockSettings};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_vulkan::VulkanAppExt;
use nightshade_vulkan::backend::{
    VulkanRenderer, install_scene_pass, pass_setup, pass_with_context,
};
use pass::BoidsVulkanPass;

pub struct BoidsVulkanPlugin;

impl Plugin for BoidsVulkanPlugin {
    fn build(&self, app: &mut App) {
        app.install_vulkan_pass(install_boids_pass);
        app.on_vulkan_pre_render(hand_over_flock);
    }
}

fn install_boids_pass(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match pass::create_pass(&setup, MAX_BOID_COUNT as u32) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the boids pass was not installed: {error}"),
    }
}

fn hand_over_flock(
    mut renderer: nightshade_vulkan::pre_render::Renderer,
    time: Res<Time>,
    mut flock: ResMut<BoidsGpuResources>,
) {
    let delta_time = time.delta_time;
    let (count, visible, seeded) = (flock.count, flock.visible, flock.seeded);
    let bounds = bounds_for(count);

    let Some((context, pass)) = pass_with_context(&mut renderer, BOIDS_GPU_PASS) else {
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<BoidsVulkanPass>() else {
        return;
    };

    if !seeded {
        if let Err(error) = pass::seed(pass, context, count as u32, bounds) {
            tracing::error!("the flock was not seeded: {error}");
            return;
        }
        flock.seeded = true;
    }

    pass::hand_over(
        pass,
        FlockSettings {
            delta_time,
            bounds,
            count: count as u32,
            visible,
            ..Default::default()
        },
    );
}
