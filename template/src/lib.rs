use nightshade::prelude::*;

#[derive(Default)]
pub struct Template;

impl State for Template {
    fn title(&self) -> &str {
        "Template"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;

        spawn_sun(world);

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            15.0,
            0.0,
            std::f32::consts::FRAC_PI_4,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Template").show(ui_context, |_ui| {});
    }

    fn run_systems(&mut self, world: &mut World) {
        camera_controllers_system(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    launch_android(app, Template).unwrap();
}
