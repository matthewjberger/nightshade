use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Template)?;
    Ok(())
}

#[derive(Default)]
struct Template;

impl State for Template {
    fn title(&self) -> &str {
        "Template"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.show_skybox = true;

        let camera_position = Vec3::new(0.0, 2.0, 10.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Template").show(ui_context, |ui| {
            ui.heading("Template");
        });
    }

    fn handle_event(&mut self, _world: &mut World, message: &Message) {
        match message {
            Message::Input { event } => {
                log::debug!("Input event: {:?}", event);
            }
            Message::App { type_name, .. } => {
                log::debug!("App event: {}", type_name);
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}
