use crate::data::look::FOG_START;
use crate::data::settings::Settings;
use crate::resources::{GridlockResources, Player};
use nightshade::prelude::*;
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_config::contract::Config;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_render::data::config::RendererState;

pub fn apply(
    config: Res<Config<Settings>>,
    player: Res<Player>,
    mut resources: ResMut<GridlockResources>,
    mut settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut cameras: Query<(&mut ThirdPersonCamera,)>,
) {
    if resources.applied_revision == Some(config.revision()) {
        return;
    }
    resources.applied_revision = Some(config.revision());
    if let Some(fog) = settings.fog.as_mut() {
        fog.start = FOG_START;
        fog.end = config.number(Settings::FogReach).max(FOG_START + 1.0);
    }
    settings.bloom_intensity = config.number(Settings::Bloom);
    settings.color_grading.contrast = config.number(Settings::Contrast);
    settings.color_grading.saturation = config.number(Settings::Saturation);
    settings.color_grading.vignette_intensity = config.number(Settings::Vignette);
    renderer_state.day_night.speed = config.number(Settings::DaySpeed);
    if let Some(camera) = player.walk_camera {
        cameras.for_entity(camera, |(camera,)| {
            camera.target_distance = config
                .number(Settings::WalkDistance)
                .clamp(camera.distance_min, camera.distance_max);
            camera.target_pitch = config
                .number(Settings::WalkPitch)
                .clamp(camera.pitch_lower_limit, camera.pitch_upper_limit);
            camera.height_offset = config.number(Settings::WalkHead);
        });
    }
}
