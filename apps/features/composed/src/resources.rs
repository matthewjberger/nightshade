use nightshade::prelude::Entity;
use nightshade_weather::Sky;

#[derive(Default)]
pub struct Meadow {
    pub camera: Option<Entity>,
    pub watched: Option<Entity>,
    pub sky: Sky,
}
