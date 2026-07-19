//! Image-based lighting maintenance: loading HDR skyboxes, capturing the
//! procedural atmosphere into IBL cubemaps, and blending between day-night
//! snapshot brackets. Every capture fans the resulting views out to the mesh
//! and skinned mesh passes and records them in [`RenderInputs::ibl_views`].

#[cfg(feature = "hdr")]
use crate::config::IblViews;
use crate::wgpu::WgpuRenderer;
use crate::wgpu::passes;
use crate::wgpu::render_configs::RenderInputs;

#[cfg(feature = "hdr")]
fn apply_ibl_views(
    renderer: &mut WgpuRenderer,
    ibl_views: &mut IblViews,
    irradiance_view: wgpu::TextureView,
    prefiltered_view: wgpu::TextureView,
) {
    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
        mesh_pass.update_ibl_textures(
            renderer.brdf_lut_view.clone(),
            irradiance_view.clone(),
            prefiltered_view.clone(),
        );
    }

    if let Some(skinned_mesh_pass) =
        crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
    {
        skinned_mesh_pass.set_ibl_textures(
            renderer.brdf_lut_view.clone(),
            irradiance_view.clone(),
            prefiltered_view.clone(),
        );
    }

    ibl_views.brdf_lut_view = Some(renderer.brdf_lut_view.clone());
    ibl_views.irradiance_view = Some(irradiance_view);
    ibl_views.prefiltered_view = Some(prefiltered_view);
}

/// Loads HDR skybox images through the sky pass and fans the captured IBL
/// views out to the mesh and skinned mesh passes and `ibl_views`.
#[cfg(feature = "hdr")]
pub fn load_hdr_skyboxes(
    renderer: &mut WgpuRenderer,
    ibl_views: &mut IblViews,
    hdr_skybox_data: Vec<Vec<u8>>,
) {
    if hdr_skybox_data.is_empty() {
        return;
    }
    let mut captured = None;

    if let Some(sky_pass) = crate::wgpu::pass_access::sky_pass_mut(&mut renderer.graph) {
        for hdr_data in hdr_skybox_data {
            captured = Some(sky_pass.load_hdr_skybox(&renderer.device, &renderer.queue, &hdr_data));
        }
    }

    if let Some((irradiance_view, prefiltered_view)) = captured {
        apply_ibl_views(renderer, ibl_views, irradiance_view, prefiltered_view);
    }
}

/// Keeps the procedural-atmosphere IBL in sync with the frame's settings:
/// re-captures the cubemaps when the atmosphere changes, and for the day-night
/// atmosphere captures the hour-bracket snapshots once the cycle starts moving.
#[cfg(feature = "hdr")]
pub fn sync_procedural_ibl(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    use crate::config::Atmosphere;
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
        let time = if atmosphere == Atmosphere::DayNight {
            hour
        } else {
            0.0
        };

        let mut captured = None;
        if let Some(sky_pass) = crate::wgpu::pass_access::sky_pass_mut(&mut renderer.graph) {
            captured = sky_pass.capture_procedural_atmosphere(
                &renderer.device,
                &renderer.queue,
                atmosphere,
                time,
            );
        }
        if let Some((irradiance_view, prefiltered_view)) = captured {
            apply_ibl_views(
                renderer,
                &mut inputs.ibl_views,
                irradiance_view,
                prefiltered_view,
            );
        }
    } else if atmosphere == Atmosphere::DayNight
        && !renderer.frame_state.captured_day_night_snapshots
        && hour != renderer.frame_state.captured_ibl_hour
    {
        renderer.frame_state.captured_day_night_snapshots = true;
        let hours = inputs.scene.day_night.ibl_snapshot_hours.clone();
        if let Some(sky_pass) = crate::wgpu::pass_access::sky_pass_mut(&mut renderer.graph) {
            sky_pass.capture_ibl_snapshots(&renderer.device, &renderer.queue, atmosphere, &hours);
        }
    }
}

/// Selects the day-night IBL snapshot bracket for the current hour and rebinds
/// the mesh and skinned mesh passes with the pair, blended by the fractional
/// position between the two snapshots. Writes the blend factor into
/// `settings.ibl_blend_factor` so the shading uniforms match the bound pair.
pub fn apply_ibl_bracket_blend(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    let raw_hour = inputs.scene.day_night.hour;
    let hour = raw_hour - (raw_hour / 24.0).floor() * 24.0;
    let world_id = inputs.world_id;
    let mut bracket_views: Option<(
        wgpu::TextureView,
        wgpu::TextureView,
        wgpu::TextureView,
        wgpu::TextureView,
        f32,
    )> = None;

    if let Some(sky_pass) = crate::wgpu::pass_access::sky_pass_mut(&mut renderer.graph)
        && let Some((index_a, index_b, blend)) =
            sky_pass.select_ibl_bracket(inputs.settings.atmosphere, hour)
    {
        let irr_a = sky_pass.ibl_snapshots[index_a]
            .1
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });
        let pref_a = sky_pass.ibl_snapshots[index_a]
            .2
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });
        let irr_b = sky_pass.ibl_snapshots[index_b]
            .1
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });
        let pref_b = sky_pass.ibl_snapshots[index_b]
            .2
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });
        bracket_views = Some((irr_a, pref_a, irr_b, pref_b, blend));
    }

    if let Some((irr_a, pref_a, irr_b, pref_b, blend)) = bracket_views {
        inputs.settings.ibl_blend_factor = blend;

        if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
            mesh_pass.update_ibl_textures_blended(
                renderer.brdf_lut_view.clone(),
                irr_a.clone(),
                pref_a.clone(),
                irr_b.clone(),
                pref_b.clone(),
                blend,
            );
        }

        if let Some(skinned_mesh_pass) =
            crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
        {
            skinned_mesh_pass.update_ibl_textures_blended_for_world(
                world_id,
                passes::geometry::BlendedIblViews {
                    brdf_lut: renderer.brdf_lut_view.clone(),
                    irradiance_a: irr_a.clone(),
                    prefiltered_a: pref_a.clone(),
                    irradiance_b: irr_b,
                    prefiltered_b: pref_b,
                    blend_factor: blend,
                },
            );
        }

        inputs.ibl_views.brdf_lut_view = Some(renderer.brdf_lut_view.clone());
        inputs.ibl_views.irradiance_view = Some(irr_a);
        inputs.ibl_views.prefiltered_view = Some(pref_a);
    }
}
