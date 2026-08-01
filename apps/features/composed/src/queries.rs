use nalgebra_glm::Vec2;
use nightshade::ecs::Storage;
use nightshade_weather::Sky;

pub fn meadow_height(_storage: &Storage, _spot: Vec2) -> Option<f32> {
    Some(0.0)
}

pub fn sky_line(sky: Sky) -> &'static str {
    match sky {
        Sky::Clear => "The sky is clear.",
        Sky::Cloudy => "Cloud is coming over.",
        Sky::Rain => "Rain starts on the meadow.",
        Sky::Storm => "A storm breaks.",
    }
}
