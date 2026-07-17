use crate::ecs::register_template_components;
use nightshade::prelude::*;

/// Startup: registers the app member world and builds the opening scene, a
/// grid on a nebula sky lit by a sun, viewed through an orbit camera.
pub fn initialize(world: &mut World) {
    world.ecs.add_world_at(GAME, register_template_components());

    world.res_mut::<DebugDraw>().show_grid = true;
    world.res_mut::<RenderSettings>().atmosphere = Atmosphere::Nebula;

    spawn_sun(world);

    let camera_entity = spawn_pan_orbit_camera(
        world,
        Vec3::new(0.0, 0.0, 0.0),
        15.0,
        0.0,
        std::f32::consts::FRAC_PI_4,
        "Main Camera".to_string(),
    );
    world.res_mut::<ActiveCamera>().0 = Some(camera_entity);
}
