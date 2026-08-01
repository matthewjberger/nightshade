use crate::data::Settings;
use nalgebra_glm::Vec3;
use nightshade_config::prelude::Config;

pub fn shade(config: &Config<Settings>, key: Settings) -> Vec3 {
    let channels = config.color(key);
    Vec3::new(channels[0], channels[1], channels[2])
}
