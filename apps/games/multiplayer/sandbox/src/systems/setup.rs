use crate::commands::level;
use crate::commands::players::spawn_driven;
use crate::data::assets::MODEL_COUNT;
use crate::queries::spawn_of;
use crate::resources::{Models, Session, Wire};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::{ActiveCamera, Camera, PerspectiveCamera, Projection};
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_gltf::Loading;
use nightshade_graphics::prelude::{Atmosphere, RenderSettings, RendererState};
use nightshade_platform::prelude::Window;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn prepare(mut commands: Commands) {
    level::build(&mut commands);
}

pub fn initialize(
    wire: Res<Wire>,
    mut session: ResMut<Session>,
    mut models: ResMut<Models>,
    mut settings: ResMut<RenderSettings>,
    mut renderer: ResMut<RendererState>,
    mut window: ResMut<Window>,
    mut loading: ResMut<Loading>,
    mut active: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    window.title = theme::TITLE.to_string();
    settings.atmosphere = Atmosphere::DayNight;
    renderer.day_night.hour = 10.0;
    renderer.day_night.auto_cycle = false;

    for index in 0..MODEL_COUNT {
        models.handles[index] = Some(crate::commands::models::request_model(&mut loading, index));
        models.requested[index] = true;
    }

    level::build(&mut commands);

    let body = spawn_driven(&mut commands, wire.peer, 0, spawn_of(0));
    session.local = Some(body);
    session.bodies.insert(wire.peer, body);

    let camera = commands.spawn((
        Name("Camera".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: theme::CAMERA_FOV_DEGREES.to_radians(),
                z_far: Some(2000.0),
                z_near: 0.1,
            }),
            smoothing: None,
        },
        ThirdPersonCamera {
            follow_target: Some(body),
            distance: theme::CAMERA_DISTANCE,
            target_distance: theme::CAMERA_DISTANCE,
            pitch: theme::CAMERA_PITCH,
            target_pitch: theme::CAMERA_PITCH,
            height_offset: 1.4,
            ..Default::default()
        },
    ));
    active.0 = Some(camera);
    session.camera = Some(camera);
}
