use nalgebra_glm::Vec4;

pub fn blend_color(from: Vec4, to: Vec4, t: f32) -> Vec4 {
    if t <= 0.0 {
        return from;
    }
    if t >= 1.0 {
        return to;
    }

    let from_hsla = crate::data::color::Hsla::from_rgba(from);
    let to_hsla = crate::data::color::Hsla::from_rgba(to);

    crate::data::color::Hsla {
        hue: crate::data::color::lerp_hue(from_hsla.hue, to_hsla.hue, t),
        saturation: from_hsla.saturation + (to_hsla.saturation - from_hsla.saturation) * t,
        lightness: from_hsla.lightness + (to_hsla.lightness - from_hsla.lightness) * t,
        alpha: from_hsla.alpha + (to_hsla.alpha - from_hsla.alpha) * t,
    }
    .to_rgba()
}
