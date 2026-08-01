use crate::data::BannerStyle;
use crate::resources::{Banner, BannerHandles, BannerShown};
use crate::systems::frame::show_the_banner;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct BannerPlugin {
    pub style: BannerStyle,
}

impl Plugin for BannerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.style);
        app.insert_resource(Banner::default());
        app.insert_resource(BannerHandles::default());
        app.insert_resource(BannerShown::default());
        app.add_system(Stage::Update, show_the_banner);
    }
}
