use crate::resources::{Style, TextStyleResources, UiHandles};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_text::contract::{TextAlignment, VerticalAlignment};
use nightshade_ui::commands::widgets::world_layout::UiLayoutNodes;
use nightshade_ui::data::state::UiBase;
use nightshade_ui::data::units::{Ab, Rl};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn build(context: UiBuildContext, style: Style) -> UiHandles {
    let mut tree = UiTreeBuilder::new(context);
    let panel = tree.add_floating_panel("text_style", "Text style", theme::PANEL_RECT);
    let content = tree.panel_content(panel).expect("panel content");
    let mut handles = UiHandles {
        panel,
        ..Default::default()
    };
    tree.in_parent(content, |tree| {
        let font_size = tree.active_theme().font_size * 1.4;
        let heading_slot = tree
            .text_state_mut()
            .cache
            .add_text("This heading takes the border too");
        handles.heading = tree
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
            .with_text_slot(heading_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_text_outline(style.outline_color, style.outline_width)
            .with_theme_color::<UiBase>(ThemeColor::TextAccent)
            .entity();
        tree.add_label("Drag to orbit, scroll to zoom. F1 hides this panel.");
        tree.add_label("Outline width, in pixels");
        handles.width_slider = tree.add_slider(0.0, theme::WIDEST_OUTLINE, style.outline_width);
        tree.add_label("Edge softness, in pixels");
        handles.smoothing_slider = tree.add_slider(0.0, theme::SOFTEST_EDGE, style.smoothing);
        tree.add_label("Outline red");
        handles.red_slider = tree.add_slider(0.0, 1.0, style.outline_color.x);
        tree.add_label("Outline green");
        handles.green_slider = tree.add_slider(0.0, 1.0, style.outline_color.y);
        tree.add_label("Outline blue");
        handles.blue_slider = tree.add_slider(0.0, 1.0, style.outline_color.z);
        tree.add_label("Scale of the text in the world");
        handles.scale_slider = tree.add_slider(0.5, theme::LARGEST_SCALE, style.scale);
        handles.billboard_toggle = tree.add_checkbox("Title faces the camera", style.billboard);
    });
    tree.finish();
    handles
}

pub fn poll(
    mut resources: ResMut<TextStyleResources>,
    mut widgets: Query<(Option<&UiSliderData>, Option<&UiCheckboxData>)>,
) {
    let resources = &mut *resources;
    let handles = resources.handles;
    let mut slider_value = |entity: Entity, current: f32| {
        let mut value = current;
        widgets.for_entity(entity, |(slider, _)| {
            if let Some(slider) = slider {
                value = slider.value;
            }
        });
        value
    };
    let style = &mut resources.style;
    style.outline_width = slider_value(handles.width_slider, style.outline_width);
    style.smoothing = slider_value(handles.smoothing_slider, style.smoothing);
    style.outline_color.x = slider_value(handles.red_slider, style.outline_color.x);
    style.outline_color.y = slider_value(handles.green_slider, style.outline_color.y);
    style.outline_color.z = slider_value(handles.blue_slider, style.outline_color.z);
    style.scale = slider_value(handles.scale_slider, style.scale);
    widgets.for_entity(handles.billboard_toggle, |(_, toggle)| {
        if let Some(toggle) = toggle {
            style.billboard = toggle.value;
        }
    });
}

pub fn drain_keyboard(
    resources: Res<TextStyleResources>,
    reader: EventReader<AppEvent>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut nodes: UiLayoutNodes<'_, '_>,
) {
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key: KeyCode::F1,
            state: KeyState::Pressed,
        } = event
        {
            ui_toggle_visible(
                &mut nodes,
                &mut dirty,
                &mut commands,
                resources.handles.panel,
            );
        }
    }
}
