use crate::components::Spin;
use nightshade::prelude::*;

/// Startup: builds the opening scene and spawns a cube carrying a `Spin` game
/// component for the update system to rotate. The resources it configures come
/// in as `ResMut` params; the world is for spawning entities.
pub fn initialize(
    mut debug_draw: ResMut<DebugDraw>,
    mut render_settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    world: &mut World,
) {
    debug_draw.show_grid = true;
    render_settings.atmosphere = Atmosphere::Nebula;

    spawn_sun(world);

    let camera_entity = spawn_pan_orbit_camera(
        world,
        Vec3::new(0.0, 0.0, 0.0),
        15.0,
        0.0,
        std::f32::consts::FRAC_PI_4,
        "Main Camera".to_string(),
    );
    active_camera.0 = Some(camera_entity);

    let cube = spawn_cube_at(world, Vec3::new(0.0, 0.5, 0.0));
    world.set(cube, Spin { speed: 1.0 });
}
