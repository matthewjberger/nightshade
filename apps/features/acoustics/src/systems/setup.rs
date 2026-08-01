use crate::commands::lay_out_the_building;
use crate::data::{
    BUILDING, CAMERA_PITCH, CAMERA_PLACE, CAMERA_YAW, CHEST_DROP, CHEST_SIZE, CLIP, CLUTTER,
    DOOR_LEAF, EMITTER_REACH, HEAD_SIZE, RAIN, RAIN_REACH, SOURCE_PLACE, Slab,
};
use crate::resources::AcousticsDemo;
use crate::systems::panel;
use nightshade::prelude::*;
use nightshade_acoustics::contract::{
    AcousticEmitter, AcousticListener, AcousticsGrid, EmitterVoice,
};
use nightshade_audio::contract::{AudioBus, AudioListener, AudioSource};
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_fly_camera_at;
use nightshade_graphics::prelude::*;

pub fn initialize(
    mut demo: ResMut<AcousticsDemo>,
    mut grid: ResMut<AcousticsGrid>,
    mut debug_draw: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    debug_draw.show_grid = false;
    settings.atmosphere = Atmosphere::Sky;
    demo.shut = false;

    spawn_sun(&mut commands);

    for (index, slab) in BUILDING.iter().chain(CLUTTER.iter()).enumerate() {
        raise_slab(&mut registry, &mut commands, slab, format!("slab_{index}"));
    }
    demo.door = raise_slab(&mut registry, &mut commands, &DOOR_LEAF, "door".to_string());
    commands.set(
        demo.door,
        nightshade_scene::contract::primitives::Visibility { visible: demo.shut },
    );

    demo.source = spawn_sphere_at(&mut commands, SOURCE_PLACE);
    set_material(
        &mut registry,
        &mut commands,
        demo.source,
        "speaker".to_string(),
        Material {
            base_color: Vec4::new(0.90, 0.55, 0.15, 1.0).into(),
            emissive_factor: Vec3::new(0.6, 0.3, 0.05).into(),
            ..Default::default()
        },
    );
    commands.set(
        demo.source,
        AudioSource::new(CLIP)
            .with_looping(true)
            .with_spatial(true)
            .with_bus(AudioBus::Music)
            .with_distance(2.0, 45.0)
            .with_reverb(true),
    );
    commands.set(
        demo.source,
        AcousticEmitter::new(EMITTER_REACH)
            .with_voice(EmitterVoice::Tone)
            .with_own_reverb(),
    );

    demo.rain = commands.spawn((
        nightshade_scene::contract::LocalTransform::default(),
        nightshade_scene::contract::GlobalTransform::default(),
        AudioSource::new(RAIN)
            .with_looping(true)
            .with_spatial(true)
            .with_bus(AudioBus::Ambient)
            .with_distance(1.0, 30.0),
    ));
    commands.set(
        demo.rain,
        AcousticEmitter::new(RAIN_REACH * 2.0).with_voice(EmitterVoice::Step),
    );

    let camera = spawn_fly_camera_at(
        &mut commands,
        CAMERA_PLACE,
        CAMERA_YAW,
        CAMERA_PITCH,
        "Acoustics Camera".to_string(),
    );
    commands.set(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_near: 0.1,
                z_far: Some(400.0),
            }),
            smoothing: Some(Smoothing::default()),
        },
    );
    active_camera.0 = Some(camera);
    demo.camera = camera;
    demo.stand = CAMERA_PLACE;
    demo.eye = CAMERA_PLACE;

    demo.head = spawn_mesh(&mut commands, "Sphere", CAMERA_PLACE, HEAD_SIZE);
    set_material(
        &mut registry,
        &mut commands,
        demo.head,
        "head".to_string(),
        Material {
            base_color: Vec4::new(0.86, 0.74, 0.62, 1.0).into(),
            ..Default::default()
        },
    );
    commands.set(demo.head, AudioListener);
    commands.set(demo.head, AcousticListener);
    commands.set(
        demo.head,
        nightshade_scene::contract::primitives::Visibility { visible: false },
    );

    demo.chest = spawn_mesh(
        &mut commands,
        "Cylinder",
        CAMERA_PLACE - Vec3::new(0.0, CHEST_DROP, 0.0),
        CHEST_SIZE,
    );
    set_material(
        &mut registry,
        &mut commands,
        demo.chest,
        "chest".to_string(),
        Material {
            base_color: Vec4::new(0.24, 0.36, 0.52, 1.0).into(),
            ..Default::default()
        },
    );
    commands.set(
        demo.chest,
        nightshade_scene::contract::primitives::Visibility { visible: false },
    );

    demo.markers = commands.spawn((
        nightshade_render::components::lines::Lines::default(),
        nightshade_scene::contract::primitives::Visibility { visible: true },
        nightshade_scene::contract::GlobalTransform::default(),
    ));
    demo.paths = commands.spawn((
        nightshade_render::components::lines::Lines::default(),
        nightshade_scene::contract::primitives::Visibility { visible: true },
        nightshade_scene::contract::GlobalTransform::default(),
    ));
    demo.showing = true;
    demo.flowing = true;

    lay_out_the_building(&mut grid, demo.shut);

    demo.handles = panel::build(&mut build);
    demo.built = true;
}

fn raise_slab(
    registry: &mut nightshade_render::resources::material_registry::MaterialRegistry,
    commands: &mut Commands,
    slab: &Slab,
    name: String,
) -> Entity {
    let middle = (slab.low + slab.high) * 0.5;
    let size = slab.high - slab.low;
    let entity = spawn_mesh(commands, "Cube", middle, size);
    set_material(
        registry,
        commands,
        entity,
        name,
        Material {
            base_color: Vec4::new(slab.color[0], slab.color[1], slab.color[2], slab.color[3])
                .into(),
            ..Default::default()
        },
    );
    entity
}
