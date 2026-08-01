use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::WgpuRenderer;
use nightshade_wgpu::backend::render_graph_set_external_texture;
use nightshade_wgpu::wgpu;

fn cube_view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    })
}

fn publish_view(
    renderer: &mut WgpuRenderer,
    identifier: nightshade_wgpu::backend::ResourceId,
    view: wgpu::TextureView,
    size: u32,
) {
    render_graph_set_external_texture(&mut renderer.graph, identifier, None, view, size, size);
}

fn publish_ibl_views(
    renderer: &mut WgpuRenderer,
    irradiance_view: wgpu::TextureView,
    prefiltered_view: wgpu::TextureView,
) {
    let irradiance = renderer.targets.ibl_irradiance;
    let prefiltered = renderer.targets.ibl_prefiltered;
    publish_view(renderer, irradiance, irradiance_view.clone(), 1);
    publish_view(renderer, prefiltered, prefiltered_view.clone(), 1);
    let irradiance_b = renderer.targets.ibl_irradiance_b;
    let prefiltered_b = renderer.targets.ibl_prefiltered_b;
    publish_view(renderer, irradiance_b, irradiance_view, 1);
    publish_view(renderer, prefiltered_b, prefiltered_view, 1);
}

fn load_hdr_skyboxes(renderer: &mut WgpuRenderer, hdr_skybox_data: Vec<Vec<u8>>) {
    if hdr_skybox_data.is_empty() {
        return;
    }
    let mut captured = None;

    let (graph, build) = renderer.graph_and_pass_build();
    if let Some(sky_pass) = crate::systems::access::sky_pass_mut(graph) {
        for hdr_data in hdr_skybox_data {
            captured = Some(sky_pass.load_hdr_skybox(build, build.queue, &hdr_data));
        }
    }

    if let Some((irradiance_view, prefiltered_view)) = captured {
        publish_ibl_views(renderer, irradiance_view, prefiltered_view);
    }
}

fn sync_procedural_ibl(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    use nightshade_render::data::config::Atmosphere;
    let atmosphere = inputs.settings.atmosphere;
    if !atmosphere.is_procedural() {
        renderer.frame_state.captured_ibl_atmosphere = None;
        return;
    }
    let hour = inputs.scene.day_night.hour;
    if renderer.frame_state.captured_ibl_atmosphere != Some(atmosphere) {
        renderer.frame_state.captured_ibl_atmosphere = Some(atmosphere);
        renderer.frame_state.captured_ibl_hour = hour;
        renderer.frame_state.captured_day_night_snapshots = false;
        renderer.frame_state.captured_ibl_bracket = None;
        let time = if atmosphere == Atmosphere::DayNight {
            hour
        } else {
            0.0
        };

        let mut captured = None;
        let (graph, build) = renderer.graph_and_pass_build();
        if let Some(sky_pass) = crate::systems::access::sky_pass_mut(graph) {
            captured = sky_pass.capture_procedural_atmosphere(build, build.queue, atmosphere, time);
        }
        if let Some((irradiance_view, prefiltered_view)) = captured {
            publish_ibl_views(renderer, irradiance_view, prefiltered_view);
        }
    } else if atmosphere == Atmosphere::DayNight
        && !renderer.frame_state.captured_day_night_snapshots
        && hour != renderer.frame_state.captured_ibl_hour
    {
        renderer.frame_state.captured_day_night_snapshots = true;
        let hours = inputs.scene.day_night.ibl_snapshot_hours.clone();
        let (graph, build) = renderer.graph_and_pass_build();
        if let Some(sky_pass) = crate::systems::access::sky_pass_mut(graph) {
            sky_pass.capture_ibl_snapshots(build, build.queue, atmosphere, &hours);
        }
    }
}

fn apply_ibl_bracket_blend(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    let raw_hour = inputs.scene.day_night.hour;
    let hour = raw_hour - (raw_hour / 24.0).floor() * 24.0;
    let mut bracket = None;

    if let Some(sky_pass) = crate::systems::access::sky_pass_mut(&mut renderer.graph)
        && let Some((index_a, index_b, blend)) =
            sky_pass.select_ibl_bracket(inputs.settings.atmosphere, hour)
    {
        let views = if renderer.frame_state.captured_ibl_bracket == Some((index_a, index_b)) {
            None
        } else {
            Some((
                cube_view(&sky_pass.ibl_snapshots[index_a].1),
                cube_view(&sky_pass.ibl_snapshots[index_a].2),
                cube_view(&sky_pass.ibl_snapshots[index_b].1),
                cube_view(&sky_pass.ibl_snapshots[index_b].2),
            ))
        };
        bracket = Some((index_a, index_b, blend, views));
    }

    let Some((index_a, index_b, blend, views)) = bracket else {
        return;
    };
    inputs.settings.ibl_blend_factor = blend;

    let Some((irradiance_a, prefiltered_a, irradiance_b, prefiltered_b)) = views else {
        return;
    };
    renderer.frame_state.captured_ibl_bracket = Some((index_a, index_b));

    let targets = (
        renderer.targets.ibl_irradiance,
        renderer.targets.ibl_prefiltered,
        renderer.targets.ibl_irradiance_b,
        renderer.targets.ibl_prefiltered_b,
    );
    publish_view(renderer, targets.0, irradiance_a, 1);
    publish_view(renderer, targets.1, prefiltered_a, 1);
    publish_view(renderer, targets.2, irradiance_b, 1);
    publish_view(renderer, targets.3, prefiltered_b, 1);
}

pub fn sync_image_lighting(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    let hdr_skybox_data = std::mem::take(&mut inputs.hdr_skybox_data);
    load_hdr_skyboxes(renderer, hdr_skybox_data);
    sync_procedural_ibl(renderer, inputs);
    apply_ibl_bracket_blend(renderer, inputs);
}
