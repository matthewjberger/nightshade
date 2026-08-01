use crate::contract::{Action, Actions, Binding, Bindings, DEFAULT_PROFILE};
use crate::resources::BindingsFile;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_gamepad::contract::POLL_SYSTEM;
use nightshade_store::prelude::{store_read_text, store_write_text};
use std::marker::PhantomData;

pub struct InputPlugin<A: Action> {
    bindings: Bindings<A>,
    filling: String,
    file: Option<(&'static str, &'static str)>,
    action: PhantomData<fn() -> A>,
}

impl<A: Action> Default for InputPlugin<A> {
    fn default() -> Self {
        Self {
            bindings: Bindings::default(),
            filling: DEFAULT_PROFILE.to_string(),
            file: None,
            action: PhantomData,
        }
    }
}

impl<A: Action> InputPlugin<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn profile(mut self, name: &str) -> Self {
        self.filling = name.to_string();
        crate::commands::open_profile(&mut self.bindings, name);
        self
    }

    pub fn bind(mut self, action: A, binding: Binding) -> Self {
        let profile = self.filling.clone();
        crate::commands::bind(&mut self.bindings, &profile, action, binding);
        self
    }

    pub fn only(mut self, action: A, binding: Binding) -> Self {
        let profile = self.filling.clone();
        crate::commands::bind_only(&mut self.bindings, &profile, action, binding);
        self
    }

    pub fn wearing(mut self, name: &str) -> Self {
        crate::commands::wear(&mut self.bindings, name);
        self
    }

    pub fn persist(mut self, folder: &'static str, name: &'static str) -> Self {
        self.file = Some((folder, name));
        self
    }
}

impl<A: Action> Plugin for InputPlugin<A> {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::input_capture::InputCapture => "PlatformPlugin",
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        let mut bindings = Bindings::<A> {
            profiles: self.bindings.profiles.clone(),
            worn: self.bindings.worn.clone(),
            revision: 0,
        };
        if let Some((folder, name)) = self.file {
            match store_read_text(folder, name) {
                Some(text) => crate::commands::bindings_apply_text(&mut bindings, &text),
                None => {
                    store_write_text(folder, name, &crate::queries::bindings_text(&bindings));
                }
            }
            let mut file = BindingsFile::<A>::new(folder, name);
            file.saved_revision = bindings.revision;
            app.insert_resource(file);
            app.add_system(Stage::Last, crate::systems::persist::keep_the_bindings::<A>);
        }
        install::<A>(app, bindings);
        register::<A>(app);
    }
}

fn install<A: Action>(app: &mut App, bindings: Bindings<A>) {
    app.insert_resource(bindings);
    app.insert_resource(Actions::<A>::default());
}

fn register<A: Action>(app: &mut App) {
    app.add_system(Stage::First, crate::systems::frame::read_the_actions::<A>);
    app.stages.stage_mut(Stage::First.name()).order(
        POLL_SYSTEM,
        std::any::type_name_of_val(&crate::systems::frame::read_the_actions::<A>),
    );
}
