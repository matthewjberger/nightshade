use crate::resources::{DialogueCard, DialogueStyle};
use crate::systems::frame::show_the_conversation;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct UiDialoguePlugin {
    pub style: DialogueStyle,
}

impl Plugin for UiDialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueStyle {
            width: self.style.width,
            height: self.style.height,
            up: self.style.up,
            name_size: self.style.name_size,
            line_size: self.style.line_size,
            background: self.style.background,
            border: self.style.border,
            name_tint: self.style.name_tint,
            line_tint: self.style.line_tint,
        });
        app.insert_resource(DialogueCard::default());
        app.add_system(Stage::LateUpdate, show_the_conversation);
    }
}
