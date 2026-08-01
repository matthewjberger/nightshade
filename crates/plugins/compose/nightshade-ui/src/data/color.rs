use nalgebra_glm;
use nalgebra_glm::Vec4;

use crate::data::state::{UiBase, UiStateTrait};

fn hue_from_rgb(red: f32, green: f32, blue: f32, max: f32, delta: f32) -> f32 {
    let sextant = if (max - red).abs() < f32::EPSILON {
        let segment = (green - blue) / delta;
        if segment < 0.0 {
            segment + 6.0
        } else {
            segment
        }
    } else if (max - green).abs() < f32::EPSILON {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    sextant * 60.0
}

fn rgb_base_from_hue_chroma(hue: f32, chroma: f32) -> (f32, f32, f32) {
    let hue_segment = hue / 60.0;
    let secondary = chroma * (1.0 - (hue_segment % 2.0 - 1.0).abs());
    match hue_segment as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsla {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub alpha: f32,
}

impl Hsla {
    pub fn from_rgba(rgba: Vec4) -> Self {
        let red = rgba.x;
        let green = rgba.y;
        let blue = rgba.z;
        let alpha = rgba.w;

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let delta = max - min;

        let lightness = (max + min) / 2.0;

        if delta < f32::EPSILON {
            return Self {
                hue: 0.0,
                saturation: 0.0,
                lightness,
                alpha,
            };
        }

        let saturation = if lightness > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        let hue = hue_from_rgb(red, green, blue, max, delta);

        Self {
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    pub fn to_rgba(&self) -> Vec4 {
        if self.saturation < f32::EPSILON {
            return Vec4::new(self.lightness, self.lightness, self.lightness, self.alpha);
        }

        let chroma = (1.0 - (2.0 * self.lightness - 1.0).abs()) * self.saturation;
        let (red, green, blue) = rgb_base_from_hue_chroma(self.hue, chroma);

        let match_value = self.lightness - chroma / 2.0;
        Vec4::new(
            red + match_value,
            green + match_value,
            blue + match_value,
            self.alpha,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsva {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub alpha: f32,
}

impl Hsva {
    pub fn from_rgba(rgba: Vec4) -> Self {
        let red = rgba.x;
        let green = rgba.y;
        let blue = rgba.z;
        let alpha = rgba.w;

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let delta = max - min;

        let value = max;

        if delta < f32::EPSILON {
            return Self {
                hue: 0.0,
                saturation: 0.0,
                value,
                alpha,
            };
        }

        let saturation = delta / max;

        let hue = hue_from_rgb(red, green, blue, max, delta);

        Self {
            hue,
            saturation,
            value,
            alpha,
        }
    }

    pub fn to_rgba(&self) -> Vec4 {
        if self.saturation < f32::EPSILON {
            return Vec4::new(self.value, self.value, self.value, self.alpha);
        }

        let chroma = self.value * self.saturation;
        let (red, green, blue) = rgb_base_from_hue_chroma(self.hue, chroma);

        let match_value = self.value - chroma;
        Vec4::new(
            red + match_value,
            green + match_value,
            blue + match_value,
            self.alpha,
        )
    }
}

pub(crate) fn lerp_hue(from: f32, to: f32, factor: f32) -> f32 {
    let diff = to - from;
    let adjusted = if diff > 180.0 {
        diff - 360.0
    } else if diff < -180.0 {
        diff + 360.0
    } else {
        diff
    };
    (from + adjusted * factor).rem_euclid(360.0)
}

pub fn blend_state_colors(colors: &[Option<Vec4>], weights: &[f32]) -> Vec4 {
    let base_color = colors[UiBase::INDEX].unwrap_or(Vec4::new(1.0, 1.0, 1.0, 1.0));

    let blend_count = colors.len().min(weights.len());
    let total_non_base_weight: f32 = (1..blend_count)
        .filter(|&index| colors[index].is_some())
        .map(|index| weights[index])
        .sum();

    if total_non_base_weight < f32::EPSILON {
        return base_color;
    }

    let base_hsla = Hsla::from_rgba(base_color);
    let base_weight = (1.0 - total_non_base_weight).max(0.0);

    let mut blended_hue = base_hsla.hue;
    let mut blended_saturation = base_hsla.saturation * base_weight;
    let mut blended_lightness = base_hsla.lightness * base_weight;
    let mut blended_alpha = base_hsla.alpha * base_weight;

    let total_weight = base_weight + total_non_base_weight;

    for index in 1..blend_count {
        let weight = weights[index];
        if weight < f32::EPSILON {
            continue;
        }

        let state_color = match colors[index] {
            Some(color) => color,
            None => continue,
        };

        let state_hsla = Hsla::from_rgba(state_color);
        let normalized_weight = weight / total_weight;

        blended_hue = lerp_hue(blended_hue, state_hsla.hue, normalized_weight);
        blended_saturation += state_hsla.saturation * normalized_weight;
        blended_lightness += state_hsla.lightness * normalized_weight;
        blended_alpha += state_hsla.alpha * normalized_weight;
    }

    let result_hsla = Hsla {
        hue: blended_hue,
        saturation: blended_saturation.clamp(0.0, 1.0),
        lightness: blended_lightness.clamp(0.0, 1.0),
        alpha: blended_alpha.clamp(0.0, 1.0),
    };

    result_hsla.to_rgba()
}

pub fn color_wheel_dot_offset(
    width: f32,
    height: f32,
    dot_size: f32,
    hue_degrees: f32,
    saturation: f32,
) -> (f32, f32) {
    let center = nalgebra_glm::Vec2::new(width * 0.5, height * 0.5);
    let radius = (center.x.min(center.y)).max(1.0);
    let angle = hue_degrees.to_radians();
    let distance = saturation.clamp(0.0, 1.0) * radius;
    (
        center.x + angle.cos() * distance - dot_size * 0.5,
        center.y + angle.sin() * distance - dot_size * 0.5,
    )
}

pub fn color_wheel_hue_saturation(width: f32, height: f32, x: f32, y: f32) -> (f32, f32) {
    let center = nalgebra_glm::Vec2::new(width * 0.5, height * 0.5);
    let radius = (center.x.min(center.y)).max(1.0);
    let offset = nalgebra_glm::Vec2::new(x - center.x, y - center.y);
    let hue = offset.y.atan2(offset.x).to_degrees().rem_euclid(360.0);
    let saturation = (offset.magnitude() / radius).clamp(0.0, 1.0);
    (hue, saturation)
}
