use crate::tags::Moon;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::data::config::FogMode;
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
        auto_cycle: false,
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

pub fn dress_the_frame(settings: &mut RenderSettings) {
    settings.bloom_enabled = true;
    settings.bloom_intensity = theme::BLOOM_DAY;
    settings.bloom_threshold = 1.3;
    settings.bloom_knee = 0.5;
    settings.fog = Some(Fog {
        color: [0.74, 0.78, 0.88],
        start: theme::FOG_START,
        end: theme::FOG_END,
        mode: FogMode::Linear,
        ..Default::default()
    });
    settings.depth_of_field.enabled = true;
    settings.depth_of_field.focus_distance = theme::CAMERA_DISTANCES[1];
    settings.depth_of_field.focus_range = 7.0;
    settings.depth_of_field.max_blur_radius = 5.0;
}
