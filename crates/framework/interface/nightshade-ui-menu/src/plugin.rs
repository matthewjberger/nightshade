use crate::data::MenuStyle;
use crate::resources::Menus;
use crate::systems::follow_the_screen;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use std::marker::PhantomData;

pub struct MenuPlugin<S> {
    pub style: MenuStyle,
    screen: PhantomData<fn() -> S>,
}

impl<S> MenuPlugin<S> {
    pub fn new(style: MenuStyle) -> Self {
        Self {
            style,
            screen: PhantomData,
        }
    }
}

impl<S> Default for MenuPlugin<S> {
    fn default() -> Self {
        Self::new(MenuStyle::default())
    }
}

impl<S: Copy + PartialEq + Send + Sync + 'static> Plugin for MenuPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.style);
        app.insert_resource(Menus::<S>::default());
        app.add_system(Stage::Update, follow_the_screen::<S>);
    }
}
