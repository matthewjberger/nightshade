use crate::resources::{StoryCard, StoryStyle};
use crate::systems::frame::{show_the_story, take_the_line};
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct UiStoryPlugin {
    pub style: StoryStyle,
}

impl Plugin for UiStoryPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_story::Story => "the game",
            nightshade_story::Play => "the game",
        ]
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(StoryStyle {
            width: self.style.width,
            row_height: self.style.row_height,
            padding: self.style.padding,
            spacing: self.style.spacing,
            corner_radius: self.style.corner_radius,
            border_width: self.style.border_width,
            lines: self.style.lines,
            columns: self.style.columns,
            links: self.style.links,
            line_size: self.style.line_size,
            background: self.style.background,
            border: self.style.border,
            line_tint: self.style.line_tint,
        });
        app.insert_resource(StoryCard::default());
        app.add_system(Stage::Update, take_the_line);
        app.add_system(Stage::LateUpdate, show_the_story);
    }
}
