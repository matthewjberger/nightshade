use crate::tags::Moon;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::data::config::{DayNightState, RendererState};

pub fn start_the_clock(
    settings: &mut RenderSettings,
    renderer_state: &mut RendererState,
    sun: Entity,
    hour: f32,
) {
    settings.atmosphere = Atmosphere::DayNight;
    renderer_state.day_night = DayNightState {
        hour,
        speed: theme::DAY_SPEED,
        auto_cycle: true,
        sun_entity: Some(sun),
        ..Default::default()
    };
}

pub fn spawn_moon(commands: &mut Commands) {
    let eye = theme::MOON_POSITION;
    let moon = commands.spawn((
        Name("Moon".to_string()),
        LocalTransform {
            translation: eye,
            rotation: nightshade_camera::queries::camera_look::camera_look_rotation(
                eye,
                Vec3::new(0.0, 0.0, 0.0),
                0.0,
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Directional,
            color: theme::MOON_COLOR,
            intensity: 0.0,
            range: 100.0,
            ..Default::default()
        },
    ));
    commands.add_tag_type::<Moon>(moon);
}
