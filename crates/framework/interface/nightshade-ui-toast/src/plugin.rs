use crate::resources::{ToastCard, ToastStyle};
use crate::systems::frame::show_the_toast;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct UiToastPlugin {
    pub style: ToastStyle,
}

impl Plugin for UiToastPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ToastStyle {
            width: self.style.width,
            height: self.style.height,
            down: self.style.down,
            text_size: self.style.text_size,
            background: self.style.background,
            border: self.style.border,
            text: self.style.text,
        });
        app.insert_resource(ToastCard::default());
        app.add_system(Stage::LateUpdate, show_the_toast);
    }
}
