use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;

pub fn stand_it_up(mut active_camera: ResMut<ActiveCamera>, mut commands: Commands) {
    let camera = {
        let commands = &mut commands;

        spawn_pan_orbit_camera_at(
            commands,
            nalgebra_glm::vec3(0.0, 1.0, 0.0),
            26.0,
            0.9,
            0.6,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);
    {
        let commands = &mut commands;

        spawn_sun(commands);
    };
}
