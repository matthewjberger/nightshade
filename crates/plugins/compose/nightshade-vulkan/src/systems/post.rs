use crate::pre_render::Renderer;
use crate::renderer::post::{
    FocusParams, IndirectParams, OcclusionParams, POST_PASS, PostParams, ReflectionParams,
    ResolveParams, SmoothParams, TracedInputs, create_pass, hand_over, hand_over_traced,
};
use crate::renderer::scene::pass::{SCENE_PASS, ScenePass, traced_inputs};
use crate::renderer::{VulkanRenderer, install_post_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_render::data::config::{
    AntiAliasing, DepthOfFieldQuality, EffectiveShading, RenderSettings, RenderView, RenderViews,
    RendererState,
};

pub fn install(renderer: &mut VulkanRenderer) {
    let extent = renderer.swapchain.extent;
    let scene_view = renderer.swapchain.hdr.view;
    let setup = pass_setup(renderer);
    match create_pass(&setup, extent, scene_view) {
        Ok(pass) => install_post_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the post pass was not installed: {error}"),
    }
}

pub fn upload_post(
    mut renderer: Renderer,
    mut state: ResMut<RendererState>,
    settings: Res<RenderSettings>,
    views: Res<RenderViews>,
    time: Res<nightshade_platform::contract::time::Time>,
) {
    let shading = views
        .active
        .as_ref()
        .map(|camera| camera.effective_shading)
        .unwrap_or(views.fallback_shading);
    let delta = time.delta_time.clamp(0.0, 0.25);
    let frame_time_ms = time.delta_time * 1000.0;
    if frame_time_ms > 0.0 {
        state.adaptive_sampling.record_frame_time(frame_time_ms);
    }
    let sample_scale = state.adaptive_sampling.ssao_sample_scale.clamp(0.25, 1.0);
    let occlusion = occlusion_params(&shading, renderer.view.as_ref(), sample_scale);
    let reflection = reflection_params(&shading, renderer.view.as_ref());
    let indirect = indirect_params(&shading, renderer.view.as_ref());
    let resolve = resolve_params(&shading, &state, renderer.view.as_ref());
    let focus = focus_params(&shading, renderer.view.as_ref());
    let smooth = smooth_params(
        &shading,
        renderer.swapchain.render_extent,
        renderer.swapchain.extent,
    );
    let grading = shading.color_grading;
    let params = PostParams {
        exposure: grading.exposure * grading.exposure_compensation_ev.exp2(),
        gamma: if grading.gamma > 0.0 {
            grading.gamma
        } else {
            2.2
        },
        chromatic_aberration: grading.chromatic_aberration.clamp(0.0, 1.0),
        world_enabled: u32::from(settings.render_world_to_swapchain),
        saturation: grading.saturation,
        brightness: grading.brightness,
        contrast: grading.contrast,
        vibrance: grading.vibrance,
        bloom_intensity: shading.bloom_intensity,
        vignette_intensity: grading.vignette_intensity,
        vignette_radius: grading.vignette_radius,
        vignette_smoothness: grading.vignette_smoothness,
        tonemap: grading.tonemap_algorithm.as_u32(),
        bloom_enabled: u32::from(shading.bloom_enabled),
        occlusion_enabled: u32::from(shading.ssao_enabled),
        occlusion_only: u32::from(shading.ssao_visualization),
        reflection_enabled: u32::from(shading.ssr_enabled),
        indirect_enabled: u32::from(shading.ssgi_enabled),
        indirect_intensity: shading.ssgi_intensity,
        resolved_enabled: u32::from(shading.anti_aliasing.is_temporal()),
        focus_enabled: u32::from(
            shading.depth_of_field.enabled || shading.depth_of_field.visualize_coc,
        ),
        auto_exposure_target: if grading.auto_exposure {
            grading.auto_exposure_target
        } else {
            0.0
        },
        color_lut_weight: grading.color_lut_weight.clamp(0.0, 1.0),
        exposure_compensation: grading.exposure_compensation_ev.exp2(),
        auto_exposure_lowest: grading.auto_exposure_min_ev.exp2(),
        auto_exposure_highest: grading.auto_exposure_max_ev.exp2(),
        output_encoding: match renderer.swapchain.encoding {
            crate::renderer::swapchain::Encoding::Srgb => 0,
            crate::renderer::swapchain::Encoding::ScRgb => 1,
            crate::renderer::swapchain::Encoding::Pq => 2,
        },
        paper_white: shading.hdr_paper_white.max(1.0),
        headroom: match renderer.swapchain.encoding {
            crate::renderer::swapchain::Encoding::Srgb => 1.0,
            crate::renderer::swapchain::Encoding::ScRgb
            | crate::renderer::swapchain::Encoding::Pq => {
                let peak = renderer
                    .display
                    .map(|display| display.peak_nits)
                    .filter(|peak| *peak > 1.0)
                    .unwrap_or(shading.hdr_peak_nits);
                (peak / shading.hdr_paper_white.max(1.0)).max(1.0)
            }
        },
    };

    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == POST_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(
            pass,
            params,
            occlusion,
            reflection,
            indirect,
            resolve,
            focus,
            smooth,
            [
                grading.auto_exposure_rate,
                delta,
                DARKEST_READING,
                BRIGHTEST_READING,
            ],
            shading.ssao_blur_depth_threshold,
            shading.ssao_blur_normal_power.max(0.0),
            shading.bloom_threshold,
            shading.bloom_knee,
            shading.bloom_filter_radius,
        );
    }
    hand_over_the_rays(&mut renderer);
}

fn hand_over_the_rays(renderer: &mut VulkanRenderer) {
    if !crate::renderer::rays::traces(&renderer.context) {
        return;
    }
    let VulkanRenderer {
        context,
        passes,
        textures,
        shadows,
        environment,
        ..
    } = renderer;
    let Some((scene_layout, scene_sets)) = passes
        .iter()
        .find(|(_, pass)| pass.name() == SCENE_PASS)
        .and_then(|(_, pass)| (pass.as_ref() as &dyn std::any::Any).downcast_ref::<ScenePass>())
        .map(traced_inputs)
    else {
        return;
    };
    let inputs = TracedInputs {
        scene_layout,
        scene_sets,
        texture_layout: textures.table.layout,
        texture_set: textures.table.set,
        shadow_layout: shadows.layout,
        shadow_sets: shadows.sets.to_vec(),
        environment_layout: environment.scene_layout,
        environment_sets: environment.scene_sets.to_vec(),
    };
    let Some(pass) = passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == POST_PASS)
        .and_then(|(_, pass)| (pass.as_mut() as &mut dyn std::any::Any).downcast_mut())
    else {
        return;
    };
    hand_over_traced(pass, context, inputs);
}

fn occlusion_params(
    shading: &EffectiveShading,
    view: Option<&RenderView>,
    sample_scale: f32,
) -> OcclusionParams {
    let Some(view) = view else {
        return OcclusionParams::default();
    };
    let Some(inverse) = view.projection.try_inverse() else {
        return OcclusionParams::default();
    };
    let mut columns = [[0.0f32; 4]; 4];
    for (index, column) in columns.iter_mut().enumerate() {
        for (row, value) in column.iter_mut().enumerate() {
            *value = inverse[(row, index)];
        }
    }
    OcclusionParams {
        inverse_projection: columns,
        projection: [
            view.projection[(0, 0)],
            view.projection[(1, 1)],
            view.screen_size.0 as f32,
            view.screen_size.1 as f32,
        ],
        settings: [
            shading.ssao_radius,
            shading.ssao_bias,
            if shading.ssao_enabled || shading.ssao_visualization {
                shading.ssao_intensity
            } else {
                0.0
            },
            (shading.ssao_sample_count.clamp(4, 64) as f32 * sample_scale)
                .round()
                .max(4.0),
        ],
        traced: [if shading.ssao_traced { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
    }
}

fn reflection_params(shading: &EffectiveShading, view: Option<&RenderView>) -> ReflectionParams {
    let Some(view) = view else {
        return ReflectionParams::default();
    };
    let Some(inverse) = view.projection.try_inverse() else {
        return ReflectionParams::default();
    };
    let mut columns = [[0.0f32; 4]; 4];
    for (index, column) in columns.iter_mut().enumerate() {
        for (row, value) in column.iter_mut().enumerate() {
            *value = inverse[(row, index)];
        }
    }
    ReflectionParams {
        inverse_projection: columns,
        projection: [
            view.projection[(0, 0)],
            view.projection[(1, 1)],
            view.projection[(2, 2)],
            view.projection[(2, 3)],
        ],
        screen: [
            view.screen_size.0 as f32,
            view.screen_size.1 as f32,
            if shading.ssr_traced { 1.0 } else { 0.0 },
            shading.ssr_thickness,
        ],
        fade: [
            shading.ssr_fade_start,
            shading.ssr_fade_end,
            shading.ssr_intensity,
            if shading.ssr_enabled { 1.0 } else { 0.0 },
        ],
        march: [
            shading.ssr_max_steps.clamp(4, 256) as f32,
            shading.ssr_max_distance.max(0.01),
            shading.ssr_stride.max(1.0),
            0.0,
        ],
    }
}

fn focus_samples(quality: DepthOfFieldQuality) -> f32 {
    quality.sample_count() as f32
}

fn focus_params(shading: &EffectiveShading, view: Option<&RenderView>) -> FocusParams {
    let Some(view) = view else {
        return FocusParams::default();
    };
    let field = shading.depth_of_field;
    let running = field.enabled || field.visualize_coc;
    FocusParams {
        focus: [
            field.focus_distance,
            field.focus_range,
            field.max_blur_radius,
            focus_samples(field.quality),
        ],
        bokeh: [
            field.bokeh_threshold,
            field.bokeh_intensity,
            view.projection[(2, 2)],
            view.projection[(2, 3)],
        ],
        screen: [
            view.screen_size.0 as f32,
            view.screen_size.1 as f32,
            if running { 1.0 } else { 0.0 },
            if field.visualize_coc { 1.0 } else { 0.0 },
        ],
        tilt: [
            if field.tilt_shift_enabled { 1.0 } else { 0.0 },
            field.tilt_shift_angle,
            field.tilt_shift_center,
            field.tilt_shift_blur_amount,
        ],
        source: [
            if field.visualize_tilt_shift { 1.0 } else { 0.0 },
            if shading.anti_aliasing.is_temporal() {
                1.0
            } else {
                0.0
            },
            0.0,
            0.0,
        ],
    }
}

fn resolve_params(
    shading: &EffectiveShading,
    state: &RendererState,
    view: Option<&RenderView>,
) -> ResolveParams {
    let Some(view) = view else {
        return ResolveParams::default();
    };
    let previous = nalgebra_glm::Mat4::from_columns(&[
        state.prev_view_projection[0].into(),
        state.prev_view_projection[1].into(),
        state.prev_view_projection[2].into(),
        state.prev_view_projection[3].into(),
    ]);
    let reprojection = previous * view.inverse_view_projection;
    let mut columns = [[0.0f32; 4]; 4];
    for (index, column) in columns.iter_mut().enumerate() {
        for (row, value) in column.iter_mut().enumerate() {
            *value = reprojection[(row, index)];
        }
    }
    ResolveParams {
        reprojection: columns,
        screen: [
            view.screen_size.0 as f32,
            view.screen_size.1 as f32,
            0.0,
            0.0,
        ],
        settings: [
            1.0,
            shading.taa_blend.clamp(0.01, 0.5),
            shading.taa_sharpness.clamp(0.0, 2.0),
            0.0,
        ],
    }
}

const SMOOTH_RELATIVE_THRESHOLD: f32 = 0.125;
const SMOOTH_ABSOLUTE_THRESHOLD: f32 = 0.0312;
const SMOOTH_SUBPIXEL: f32 = 0.75;

fn smooth_params(
    shading: &EffectiveShading,
    drawn: ash::vk::Extent2D,
    screen: ash::vk::Extent2D,
) -> Option<SmoothParams> {
    let lifting = drawn.width < screen.width || drawn.height < screen.height;
    (shading.anti_aliasing == AntiAliasing::Fxaa).then(|| SmoothParams {
        texel: [
            1.0 / drawn.width.max(1) as f32,
            1.0 / drawn.height.max(1) as f32,
        ],
        relative_threshold: SMOOTH_RELATIVE_THRESHOLD,
        absolute_threshold: SMOOTH_ABSOLUTE_THRESHOLD,
        subpixel: SMOOTH_SUBPIXEL,
        snap: if lifting { 1.0 } else { 0.0 },
    })
}

const INDIRECT_DIRECTIONS: f32 = 16.0;

fn indirect_params(shading: &EffectiveShading, view: Option<&RenderView>) -> IndirectParams {
    let Some(view) = view else {
        return IndirectParams::default();
    };
    let Some(inverse) = view.projection.try_inverse() else {
        return IndirectParams::default();
    };
    let mut columns = [[0.0f32; 4]; 4];
    for (index, column) in columns.iter_mut().enumerate() {
        for (row, value) in column.iter_mut().enumerate() {
            *value = inverse[(row, index)];
        }
    }
    IndirectParams {
        inverse_projection: columns,
        projection: [
            view.projection[(0, 0)],
            view.projection[(1, 1)],
            view.projection[(2, 2)],
            view.projection[(2, 3)],
        ],
        screen: [
            view.screen_size.0 as f32,
            view.screen_size.1 as f32,
            shading.ssgi_radius,
            INDIRECT_DIRECTIONS,
        ],
        settings: [
            shading.ssgi_max_steps.clamp(1, 32) as f32,
            0.0,
            if shading.ssgi_enabled { 1.0 } else { 0.0 },
            0.0,
        ],
    }
}

const DARKEST_READING: f32 = -10.0;
const BRIGHTEST_READING: f32 = 4.0;
