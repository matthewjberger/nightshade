use crate::pre_render::Renderer;
use crate::renderer::environment::cross_the_hours;
use crate::renderer::sky::{
    ATMOSPHERE_CLOUDY, ATMOSPHERE_DAY_NIGHT, ATMOSPHERE_HDR, ATMOSPHERE_NEBULA, ATMOSPHERE_NONE,
    ATMOSPHERE_SKY, ATMOSPHERE_SPACE, ATMOSPHERE_SUNSET, GpuSky, SKY_PASS, create_pass, hand_over,
};
use crate::renderer::{VulkanRenderer, install_sky_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_render::data::config::{Atmosphere, RenderSettings, RendererState};
use nightshade_render::queries::projection::matrix_columns;

pub fn install(renderer: &mut VulkanRenderer) {
    let environment = renderer.environment.sky.view;
    let sampler = renderer.environment.sampler;
    let air = crate::renderer::scene::fog::air_binding(&renderer.air);
    let setup = pass_setup(renderer);
    match create_pass(&setup, environment, sampler, air) {
        Ok(pass) => install_sky_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the sky pass was not installed: {error}"),
    }
}

pub fn upload_sky(
    mut renderer: Renderer,
    settings: Res<RenderSettings>,
    state: Res<RendererState>,
    time: Res<nightshade_platform::contract::time::Time>,
) {
    let atmosphere = if settings.show_sky {
        atmosphere_code(settings.atmosphere)
    } else {
        ATMOSPHERE_NONE
    };
    renderer.clear_color = match settings.fog.as_ref() {
        Some(fog) => [fog.color[0], fog.color[1], fog.color[2], 1.0],
        None => settings.clear_color,
    };
    renderer.swapchain_clear = settings.swapchain_clear_color;
    let Some(view) = renderer.view.clone() else {
        return;
    };
    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == SKY_PASS)
        .and_then(|(_, pass)| (pass.as_mut() as &mut dyn std::any::Any).downcast_mut())
    else {
        return;
    };
    let (fog_color, lit_air) = settings
        .fog
        .as_ref()
        .map(|fog| {
            (
                [fog.color[0], fog.color[1], fog.color[2], 1.0],
                if fog.volumetric { 1.0 } else { 0.0 },
            )
        })
        .unwrap_or(([0.0; 4], 0.0));
    hand_over(
        pass,
        GpuSky {
            projection: matrix_columns(&view.projection),
            view: matrix_columns(&view.view),
            settings: [
                time.uptime_seconds as f32,
                state.day_night.hour,
                atmosphere as f32,
                environment_level(settings.atmosphere),
            ],
            air: [
                view.screen_size.0.max(1) as f32,
                view.screen_size.1.max(1) as f32,
                lit_air,
                0.0,
            ],
            fog_color,
        },
    );
}

pub fn light_from_the_sky(
    mut renderer: Renderer,
    settings: Res<RenderSettings>,
    state: Res<RendererState>,
    time: Res<nightshade_platform::contract::time::Time>,
) {
    let atmosphere = atmosphere_code(settings.atmosphere);
    if !painted_by_the_engine(atmosphere) {
        return;
    }
    let VulkanRenderer {
        context,
        environment,
        ..
    } = &mut *renderer;
    if let Err(error) = cross_the_hours(
        context,
        environment,
        atmosphere,
        time.uptime_seconds as f32,
        state.day_night.hour,
        &state.day_night.ibl_snapshot_hours,
    ) {
        tracing::error!("the sky was not painted into the environment: {error}");
    }
}

fn painted_by_the_engine(atmosphere: u32) -> bool {
    matches!(
        atmosphere,
        ATMOSPHERE_NONE
            | ATMOSPHERE_SKY
            | ATMOSPHERE_CLOUDY
            | ATMOSPHERE_SPACE
            | ATMOSPHERE_NEBULA
            | ATMOSPHERE_SUNSET
            | ATMOSPHERE_DAY_NIGHT
    )
}

fn atmosphere_code(atmosphere: Atmosphere) -> u32 {
    match atmosphere {
        Atmosphere::None => ATMOSPHERE_NONE,
        Atmosphere::Sky => ATMOSPHERE_SKY,
        Atmosphere::CloudySky => ATMOSPHERE_CLOUDY,
        Atmosphere::Space => ATMOSPHERE_SPACE,
        Atmosphere::Nebula => ATMOSPHERE_NEBULA,
        Atmosphere::Sunset => ATMOSPHERE_SUNSET,
        Atmosphere::DayNight => ATMOSPHERE_DAY_NIGHT,
        _ => ATMOSPHERE_HDR,
    }
}

fn environment_level(atmosphere: Atmosphere) -> f32 {
    match atmosphere {
        Atmosphere::HdrMip1 => 1.0,
        Atmosphere::HdrMip2 => 2.0,
        Atmosphere::HdrMip3 => 3.0,
        Atmosphere::HdrMip4 => 4.0,
        _ => 0.0,
    }
}
