use nightshade_camera::components::ShadingMode;
use nightshade_graphics::prelude::{Atmosphere, TonemapAlgorithm};
use nightshade_render::data::config::Fog;

use crate::resources::ViewerResources;

pub fn fog_of(viewer: &ViewerResources) -> Fog {
    Fog {
        color: viewer.view.fog_color,
        start: viewer.view.fog_start,
        end: viewer.view.fog_end,
        ..Fog::default()
    }
}

pub fn shading_mode(index: usize) -> ShadingMode {
    match index {
        1 => ShadingMode::Solid,
        2 => ShadingMode::Flat,
        3 => ShadingMode::Wireframe,
        _ => ShadingMode::Rendered,
    }
}

pub fn pbr_mode(index: usize) -> nightshade_render::data::config::PbrDebugMode {
    use nightshade_render::data::config::PbrDebugMode as Mode;
    match index {
        1 => Mode::BaseColor,
        2 => Mode::Normal,
        3 => Mode::Metallic,
        4 => Mode::Roughness,
        5 => Mode::Occlusion,
        6 => Mode::Emissive,
        _ => Mode::None,
    }
}

pub fn tonemap(index: usize) -> TonemapAlgorithm {
    match index {
        1 => TonemapAlgorithm::Reinhard,
        2 => TonemapAlgorithm::Uncharted2,
        3 => TonemapAlgorithm::AgX,
        4 => TonemapAlgorithm::Neutral,
        5 => TonemapAlgorithm::None,
        _ => TonemapAlgorithm::Aces,
    }
}

pub fn atmosphere(index: usize) -> Atmosphere {
    match index {
        0 => Atmosphere::None,
        1 => Atmosphere::Sky,
        2 => Atmosphere::CloudySky,
        3 => Atmosphere::Space,
        4 => Atmosphere::Nebula,
        5 => Atmosphere::Sunset,
        6 => Atmosphere::DayNight,
        _ => Atmosphere::Hdr,
    }
}
