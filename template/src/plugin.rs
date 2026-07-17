use crate::ecs::{TemplateResources, register_template_components};
use crate::systems::example;
use nightshade::prelude::*;

/// The game plugin. Registers the app member world, the app resources,
/// and the startup and update systems against the [`App`] builder; grow
/// your game by adding systems in `src/systems/` and registering them
/// here.
pub struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Template".to_string();
        app.insert_resource(TemplateResources::default());
        app.add_system(Stage::Startup, initialize);
        app.add_system(Stage::Update, update);
    }
}

fn initialize(world: &mut World) {
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

fn update(template_resources: &mut TemplateResources, world: &mut World) {
    example::tick(template_resources, world);

    let events = std::mem::take(&mut world.res_mut::<Input>().events);
    for event in events {
        if let AppEvent::Keyboard { key, state } = event
            && matches!((key, state), (KeyCode::KeyQ, KeyState::Pressed))
        {
            world.res_mut::<Window>().should_exit = true;
        }
    }
}
