use nightshade_render::data::config::TonemapAlgorithm;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LofiLook {
    pub render_scale: f32,
    pub snap: Option<[f32; 2]>,
    pub affine_textures: bool,
    pub anisotropy: u16,
    pub smooth_edges: bool,
    pub bloom: bool,
    pub ambient_occlusion: bool,
    pub saturation: f32,
    pub contrast: f32,
    pub vignette: f32,
    pub vignette_radius: f32,
    pub tonemap: TonemapAlgorithm,
}

impl Default for LofiLook {
    fn default() -> Self {
        clean()
    }
}

pub fn clean() -> LofiLook {
    LofiLook {
        render_scale: 1.0,
        snap: None,
        affine_textures: false,
        anisotropy: 16,
        smooth_edges: true,
        bloom: true,
        ambient_occlusion: true,
        saturation: 1.0,
        contrast: 1.0,
        vignette: 0.0,
        vignette_radius: 0.75,
        tonemap: TonemapAlgorithm::Aces,
    }
}

pub fn playstation() -> LofiLook {
    LofiLook {
        render_scale: 0.34,
        snap: Some([320.0, 240.0]),
        affine_textures: true,
        anisotropy: 1,
        smooth_edges: false,
        bloom: false,
        ambient_occlusion: false,
        saturation: 0.7,
        contrast: 1.12,
        vignette: 0.45,
        vignette_radius: 0.5,
        tonemap: TonemapAlgorithm::Reinhard,
    }
}

pub fn handheld() -> LofiLook {
    LofiLook {
        render_scale: 0.5,
        snap: Some([240.0, 160.0]),
        affine_textures: false,
        anisotropy: 1,
        smooth_edges: false,
        bloom: false,
        ambient_occlusion: false,
        saturation: 1.15,
        contrast: 1.05,
        vignette: 0.2,
        vignette_radius: 0.7,
        tonemap: TonemapAlgorithm::Reinhard,
    }
}
