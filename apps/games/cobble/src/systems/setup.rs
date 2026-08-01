use blocker_blocks::commands::install_the_blocks;
use blocker_blocks::resources::{Sheet, VoxelSettings, VoxelWorld};
use blocker_figure::resources::Hero;
use blocker_player::commands::{
    install_the_craft, spawn_crack, spawn_outline, spawn_the_camera, spawn_the_hero,
};
use blocker_player::resources::Markers;
use blocker_world::resources::Draft;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::components::light::Light;
use nightshade_render::data::config::{DayNightState, RendererState};
use nightshade_sprite::prelude::SpriteAtlases;

pub fn raise_the_blocks(
    mut mesh_cache: ResMut<MeshCache>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut sheet: ResMut<Sheet>,
    mut blocks: ResMut<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut textures: TextureLoad,
) {
    install_the_blocks(
        &mut mesh_cache,
        &mut textures,
        &mut registry,
        &mut atlases,
        &mut sheet,
        &mut blocks,
        &mut world,
    );
    install_the_craft(&mut mesh_cache);
}

pub fn initialize(
    mut render_settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut active_camera: ResMut<ActiveCamera>,
    mut draft: ResMut<Draft>,
    mut markers: ResMut<Markers>,
    mut hero: ResMut<Hero>,
    mut commands: Commands,
) {
    render_settings.atmosphere = Atmosphere::DayNight;
    render_settings.named_shadow_casters = true;
    render_settings.bloom_enabled = true;
    render_settings.bloom_intensity = blocker_weather::theme::BLOOM_DAY;
    render_settings.bloom_threshold = blocker_screen::theme::BLOOM_THRESHOLD;
    render_settings.bloom_knee = blocker_screen::theme::BLOOM_KNEE;
    draft.seed = blocker_world::data::world::SEED;

    let sun = spawn_sun(&mut commands);
    commands.set(
        sun,
        Light {
            shadow_softness: blocker_screen::theme::SUN_SHADOW_SOFTNESS,
            ..Light::default().with_shadows(0.0005)
        },
    );
    renderer_state.day_night = DayNightState {
        hour: blocker_screen::theme::START_HOUR,
        speed: blocker_screen::theme::DAY_SPEED,
        auto_cycle: true,
        sun_entity: Some(sun),
        ..Default::default()
    };

    spawn_the_camera(
        &mut commands,
        &mut active_camera,
        blocker_screen::theme::CAMERA_FOV_DEGREES,
        blocker_screen::theme::CAMERA_NEAR,
        blocker_screen::theme::CAMERA_FAR,
    );
    spawn_the_hero(&mut commands, &mut hero, None);
    markers.outline = Some(spawn_outline(&mut commands));
    markers.crack = Some(spawn_crack(&mut commands));
}
