use crate::resources::{Conversation, DialogueSettings};
use crate::systems::frame::reveal_the_line;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct DialoguePlugin {
    pub settings: DialogueSettings,
}

impl Plugin for DialoguePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueSettings {
            letters_a_second: self.settings.letters_a_second,
        });
        app.insert_resource(Conversation::default());
        app.add_system(Stage::Update, reveal_the_line);
    }
}
