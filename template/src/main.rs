use nightshade::prelude::*;

#[cfg(feature = "plugins")]
mod plugin_runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Template::default())?;
    Ok(())
}

#[derive(Default)]
struct Template {
    #[cfg(feature = "plugins")]
    plugin_runtime: Option<plugin_runtime::PluginRuntime>,
}

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

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_enabled = true;
        }

        #[cfg(feature = "plugins")]
        {
            let plugins_dir = std::path::PathBuf::from("plugins/plugins");

            match plugin_runtime::PluginRuntime::new(plugins_dir.clone()) {
                Ok(mut runtime) => {
                    tracing::info!("Loading plugins from: {:?}", plugins_dir);

                    if let Err(error) = runtime.load_plugins_from_directory(&plugins_dir) {
                        tracing::error!("Failed to load plugins: {}", error);
                    }

                    runtime.call_on_init(world);
                    self.plugin_runtime = Some(runtime);
                }
                Err(error) => {
                    tracing::error!("Failed to create plugin runtime: {}", error);
                }
            }
        }
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Template").show(ui_context, |ui| {
            ui.heading("Template");
        });
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);

        #[cfg(feature = "plugins")]
        if let Some(runtime) = &mut self.plugin_runtime {
            runtime.run_frame(world);
        }
    }

    fn handle_event(&mut self, _world: &mut World, message: &Message) {
        match message {
            Message::Input { event } => {
                tracing::debug!("Input event: {:?}", event);
            }
            Message::App { type_name, .. } => {
                tracing::debug!("App event: {}", type_name);
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }

        #[cfg(feature = "plugins")]
        if let Some(runtime) = &mut self.plugin_runtime {
            runtime.queue_keyboard_event(key_code, key_state);
        }
    }

    fn on_mouse_input(&mut self, _world: &mut World, state: ElementState, button: MouseButton) {
        #[cfg(feature = "plugins")]
        if let Some(runtime) = &mut self.plugin_runtime {
            runtime.queue_mouse_event(state, button);
        }

        let _ = (state, button);
    }
}
