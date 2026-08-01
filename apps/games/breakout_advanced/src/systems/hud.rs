use crate::data;
use crate::queries;
use crate::resources::{BreakoutAdvancedResources, Screen};
use crate::systems::screens;
use nightshade::prelude::*;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::{MenuStyle, Menus};

pub fn build_overlay(
    mut resources: ResMut<BreakoutAdvancedResources>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let readout = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, data::HUD_MARGIN)),
            Ab(vec2(data::HUD_WIDTH, data::HUD_HEIGHT)),
            Anchor::TopCenter,
        )
        .with_text("", data::HUD_TEXT_SIZE)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 1.0), 1.0)
        .color_raw::<UiBase>(Vec4::from_column_slice(&data::HUD_TEXT_COLOR))
        .without_pointer_events()
        .with_visible(false)
        .entity();

    let mut handles = screens::build(&mut tree, &style, &mut menus);
    handles.readout = Some(readout);
    tree.finish();
    resources.ui = handles;
}

pub fn show(resources: Res<BreakoutAdvancedResources>, mut commands: Commands) {
    let Some(readout) = resources.ui.readout else {
        return;
    };
    ui_set_text(
        &mut commands,
        readout,
        &queries::readout_text(&resources.round),
    );
}
