use crate::commands::{floor, slab};
use crate::resources::Warden;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_dialogue::Conversation;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn meet_the_warden(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut warden_held: ResMut<Warden>,
    mut conversation: ResMut<Conversation>,
    mut commands: Commands,
) {
    let camera = {
        let commands = &mut commands;

        spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_BACK,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);

    let warden = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let warden = slab(
            commands,
            materials,
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 2.0, 1.0),
            theme::SPEAKER_TINT,
        );
        spawn_sun(commands);
        warden
    };
    warden_held.body = Some(warden);

    nightshade_dialogue::commands::open(&mut conversation, theme::SPEAKER, &theme::LINES);
    nightshade_dialogue::commands::offer(&mut conversation, &theme::REPLIES);
}
