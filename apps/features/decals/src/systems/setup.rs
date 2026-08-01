use crate::resources::DecalsResources;
use crate::systems::decals::spawn_initial_decals;
use crate::systems::panel::{self, PanelSetup};
use crate::systems::world::{spawn_damaged_helmet, spawn_room};
use crate::theme::{
    AWESOMEFACE_PNG, AWESOMEFACE_TEXTURE, BLOOM_INTENSITY, CAMERA_DISTANCE, CAMERA_PITCH,
    CAMERA_TARGET, CAMERA_YAW, HDR_SKYBOX, HELP_TEXT, SUN_INTENSITY,
};
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::commands::procedural::load_procedural_textures;
use nightshade_graphics::prelude::*;
use nightshade_render::components::lines::Lines;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_text::commands::spawn_ui_text_with_properties;
use nightshade_text::contract::resources::TextState;
use nightshade_text::prelude::*;

pub fn initialize(
    mut debug: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut queues: ResMut<CommandQueues>,
    mut gltf_loading: ResMut<nightshade_gltf::Loading>,
    mut active_camera: ResMut<ActiveCamera>,
    mut decals_resources: ResMut<DecalsResources>,
    mut materials: ResMut<MaterialRegistry>,
    mut text_state: ResMut<TextState>,
    mut load: nightshade_graphics::resources::loading::TextureLoad,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    debug.show_grid = false;
    settings.atmosphere = Atmosphere::Hdr;
    settings.bloom_enabled = true;
    settings.bloom_intensity = BLOOM_INTENSITY;
    load_hdr_skybox(&mut queues, HDR_SKYBOX.to_vec());
    load_procedural_textures(&mut load);
    load_awesomeface_texture(&mut load);
    spawn_damaged_helmet(&mut gltf_loading);

    let setup = {
        let decals = &decals_resources;
        PanelSetup {
            decal_type: decals.current_decal_type,
            decal_size: decals.decal_size,
            emissive_strength: decals.emissive_strength,
            show_controls: decals.show_controls,
            show_preview: decals.show_preview,
            use_awesomeface: decals.use_awesomeface,
            bloom_enabled: true,
            bloom_intensity: BLOOM_INTENSITY,
        }
    };
    let emissive_strength = setup.emissive_strength;

    let (camera, preview) = {
        let commands = build.context().commands;
        let sun = spawn_sun(commands);
        commands.queue(move |storage, _| {
            if let Some(light) = storage.get_mut::<Light>(sun) {
                light.cast_shadows = false;
                light.intensity = SUN_INTENSITY;
            }
        });
        spawn_room(&mut materials, commands);
        let camera = spawn_pan_orbit_camera_at(
            commands,
            CAMERA_TARGET,
            CAMERA_DISTANCE,
            CAMERA_YAW,
            CAMERA_PITCH,
            "Decals Demo Camera".to_string(),
        );
        let preview = commands.spawn((
            Lines::default(),
            Visibility { visible: true },
            GlobalTransform::default(),
        ));
        spawn_initial_decals(commands, emissive_strength);
        spawn_ui_text_with_properties(
            &mut text_state,
            commands,
            HELP_TEXT,
            Vec2::zeros(),
            TextProperties {
                font_size: 18.0,
                color: vec4(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.02,
                outline_color: vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        (camera, preview)
    };
    let handles = panel::build(build.context(), setup);

    active_camera.0 = Some(camera);

    decals_resources.preview_lines_entity = Some(preview);
    decals_resources.handles = handles;
    decals_resources.last_decal_type = Some(decals_resources.current_decal_type);
}

fn load_awesomeface_texture(load: &mut nightshade_graphics::resources::loading::TextureLoad<'_>) {
    let rgba = match decode_to_rgba8(AWESOMEFACE_PNG) {
        Ok(decoded) => decoded,
        Err(error) => {
            tracing::error!("the awesomeface texture did not decode: {error}");
            return;
        }
    };
    let (width, height) = (rgba.width, rgba.height);
    queue_decoded_texture(
        load,
        AWESOMEFACE_TEXTURE.to_string(),
        rgba.rgba,
        width,
        height,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
    texture_cache_protect(load.cache, AWESOMEFACE_TEXTURE.to_string());
}
